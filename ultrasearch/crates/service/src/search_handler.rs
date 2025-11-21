use anyhow::Result;
use ipc::{FieldKind, QueryExpr, SearchHit, SearchRequest, SearchResponse, TermExpr, TermModifier};
use meta_index::{MetaFields, MetaIndex, open_or_create_index, open_reader};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Instant;
use tantivy::collector::{Count, TopDocs};
use tantivy::query::{BooleanQuery, Occur, Query, QueryParser, TermQuery};
use tantivy::schema::{Document, IndexRecordOption, TantivyDocument, Value};
use tantivy::{DocAddress, Score, Term};
use tracing::warn;

/// Trait for handling search requests.
pub trait SearchHandler: Send + Sync {
    fn search(&self, req: SearchRequest) -> SearchResponse;
}

/// Simple placeholder handler that returns an empty response.
#[derive(Debug, Default)]
pub struct StubSearchHandler;

impl SearchHandler for StubSearchHandler {
    fn search(&self, req: SearchRequest) -> SearchResponse {
        SearchResponse {
            id: req.id,
            hits: Vec::new(),
            total: 0,
            truncated: false,
            took_ms: 0,
            served_by: Some("service-stub".into()),
        }
    }
}

/// Handler backed by the metadata index.
pub struct MetaIndexSearchHandler {
    meta: MetaIndex,
    reader: tantivy::IndexReader,
}

impl MetaIndexSearchHandler {
    pub fn try_new(index_path: &Path) -> Result<Self> {
        let meta = open_or_create_index(index_path)?;
        let reader = open_reader(&meta)?;
        Ok(Self { meta, reader })
    }

    fn build_query(&self, expr: &QueryExpr) -> Result<Box<dyn Query>> {
        Ok(match expr {
            QueryExpr::Term(t) => self.term_query(t)?,
            // Range and other expressions are stubbed for now.
            QueryExpr::Range(_) => Box::new(BooleanQuery::new(vec![])),
            QueryExpr::Not(inner) => Box::new(BooleanQuery::new(vec![(
                Occur::MustNot,
                self.build_query(inner)?,
            )])),
            QueryExpr::And(items) if items.is_empty() => Box::new(BooleanQuery::new(vec![])),
            QueryExpr::And(items) => Box::new(BooleanQuery::new(
                items
                    .iter()
                    .map(|q| Ok((Occur::Must, self.build_query(q)?)))
                    .collect::<Result<Vec<_>>>()?,
            )),
            QueryExpr::Or(items) if items.is_empty() => Box::new(BooleanQuery::new(vec![])),
            QueryExpr::Or(items) => Box::new(BooleanQuery::new(
                items
                    .iter()
                    .map(|q| Ok((Occur::Should, self.build_query(q)?)))
                    .collect::<Result<Vec<_>>>()?,
            )),
        })
    }

    fn term_query(&self, term: &TermExpr) -> Result<Box<dyn Query>> {
        let fields = &self.meta.fields;
        let value = term.value.trim();
        if value.is_empty() {
            return Ok(Box::new(BooleanQuery::new(vec![])));
        }

        let target_fields: Vec<FieldKind> = match term.field {
            Some(f) => vec![f],
            None => vec![FieldKind::Name, FieldKind::Path],
        };

        let mut clauses = Vec::new();
        for field in target_fields {
            match field {
                FieldKind::Ext => {
                    let t = Term::from_field_text(fields.ext, value);
                    clauses.push((
                        Occur::Should,
                        Box::new(TermQuery::new(t, IndexRecordOption::WithFreqs)) as Box<dyn Query>,
                    ));
                }
                FieldKind::Name | FieldKind::Path => match term.modifier {
                    TermModifier::Prefix => {
                        let pf = if matches!(field, FieldKind::Name) {
                            fields.name
                        } else {
                            fields.path
                        };
                        // TODO: PrefixQuery removed in Tantivy 0.22; using TermQuery fallback.
                        // Consider using RegexQuery or PhraseQuery if needed.
                        let t = Term::from_field_text(pf, value);
                        clauses.push((
                            Occur::Should,
                            Box::new(TermQuery::new(t, IndexRecordOption::WithFreqs)) as Box<dyn Query>,
                        ));
                    }
                    _ => {
                        let mut parser = QueryParser::for_index(
                            &self.meta.index,
                            vec![if matches!(field, FieldKind::Name) {
                                fields.name
                            } else {
                                fields.path
                            }],
                        );
                        parser.set_conjunction_by_default();
                        if let Ok(q) = parser.parse_query(value) {
                            clauses.push((Occur::Should, q));
                        }
                    }
                },
                _ => {}
            }
        }

        Ok(Box::new(BooleanQuery::new(clauses)))
    }
}

impl SearchHandler for MetaIndexSearchHandler {
    fn search(&self, req: SearchRequest) -> SearchResponse {
        let start = Instant::now();
        let limit = req.limit.max(1) as usize;
        let offset = req.offset as usize;

        let searcher = self.reader.searcher();
        let query = match self.build_query(&req.query) {
            Ok(q) => q,
            Err(err) => {
                warn!(error = %err, "failed to build query; returning empty result");
                return StubSearchHandler.search(req);
            }
        };

        let top_k = limit.saturating_add(offset);
        let result = searcher.search(&query, &(TopDocs::with_limit(top_k), Count));

        let (hits, total): (Vec<(Score, DocAddress)>, usize) = match result {
            Ok((top, count)) => (top, count),
            Err(err) => {
                warn!(error = %err, "tantivy search failed; returning stub response");
                return StubSearchHandler.search(req);
            }
        };

        let docs_iter = hits.into_iter().skip(offset);
        let mut out = Vec::new();
        for (score, addr) in docs_iter {
            if let Ok(retrieved) = searcher.doc::<TantivyDocument>(addr) 
                && let Some(hit) = to_hit(&retrieved, &self.meta.fields, score) 
            {
                out.push(hit);
            }
        }

        SearchResponse {
            id: req.id,
            hits: out,
            total: total as u64,
            truncated: false,
            took_ms: start.elapsed().as_millis().min(u32::MAX as u128) as u32,
            served_by: None,
        }
    }
}

static HANDLER: OnceLock<Box<dyn SearchHandler>> = OnceLock::new();

pub fn set_search_handler(handler: Box<dyn SearchHandler>) {
    let _ = HANDLER.set(handler);
}

pub fn search(req: SearchRequest) -> SearchResponse {
    if let Some(h) = HANDLER.get() {
        h.search(req)
    } else {
        StubSearchHandler.search(req)
    }
}

fn to_hit<D: Document>(doc: &D, fields: &MetaFields, score: Score) -> Option<SearchHit> {
    let mut key = None;
    let mut name = None;
    let mut path = None;
    let mut ext = None;
    let mut size = None;
    let mut modified = None;

    for (field, value) in doc.iter_fields_and_values() {
        match field {
            f if f == fields.doc_key => {
                if let Some(v) = value.as_u64() {
                    key = Some(core_types::DocKey(v));
                }
            }
            f if f == fields.name => {
                if let Some(s) = value.as_str() {
                    name = Some(s.to_string());
                }
            }
            f if f == fields.path => {
                if let Some(s) = value.as_str() {
                    path = Some(s.to_string());
                }
            }
            f if f == fields.ext => {
                if let Some(s) = value.as_str() {
                    ext = Some(s.to_string());
                }
            }
            f if f == fields.size => {
                if let Some(v) = value.as_u64() {
                    size = Some(v);
                }
            }
            f if f == fields.modified => {
                if let Some(v) = value.as_i64() {
                    modified = Some(v);
                }
            }
            _ => {}
        }
    }

    key.map(|doc_key| SearchHit {
        key: doc_key,
        score,
        name,
        path,
        ext,
        size,
        modified,
        snippet: None,
    })
}
