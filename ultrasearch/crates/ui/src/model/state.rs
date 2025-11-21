use core_types::DocKey;
use gpui::{Context, Model, ModelContext, Task};
use ipc::{QueryExpr, SearchRequest, TermExpr, TermModifier};
use std::sync::Arc;
use crate::ipc::client::IpcClient;

const DEFAULT_MAX_ROWS: usize = 1000;

#[derive(Debug, Clone, PartialEq)]
pub struct ResultRow {
    pub doc_key: DocKey,
    pub name: String,
    pub path: String,
    pub ext: String,
    pub size: u64,
    pub modified_ts: i64,
    pub score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendMode {
    #[default]
    MetadataOnly,
    Mixed,
    ContentOnly,
}

#[derive(Debug, Default)]
pub struct SearchStatus {
    pub last_latency_ms: Option<u64>,
    pub shown: usize,
    pub total: usize,
    pub truncated: bool,
    pub backend_mode: BackendMode,
    pub connected: bool,
}

pub struct SearchAppModel {
    pub query: String,
    pub status: SearchStatus,
    pub results: Vec<ResultRow>,
    pub selected_index: Option<usize>,
    pub client: Arc<IpcClient>,
}

impl SearchAppModel {
    pub fn init(cx: &mut Context) -> Model<Self> {
        cx.new_model(|_cx| Self {
            query: String::new(),
            status: SearchStatus::default(),
            results: Vec::with_capacity(DEFAULT_MAX_ROWS),
            selected_index: None,
            client: Arc::new(IpcClient::new()),
        })
    }

    pub fn set_query(&mut self, text: String, cx: &mut ModelContext<Self>) {
        self.query = text;
        cx.notify();
        self.perform_search(cx);
    }

    pub fn perform_search(&mut self, cx: &mut ModelContext<Self>) -> Option<Task<()>> {
        if self.query.is_empty() {
            self.update_results(vec![], 0, 0, cx);
            return None;
        }

        let client = self.client.clone();
        let query_text = self.query.clone();
        
        Some(cx.spawn(move |model, mut cx| async move {
            // TODO: Parse query text into real AST (c00.7.3)
            // For now, just treat as prefix term on name
            let query = QueryExpr::Term(TermExpr {
                field: None,
                value: query_text,
                modifier: TermModifier::Prefix,
            });

            let req = SearchRequest {
                query,
                limit: DEFAULT_MAX_ROWS as u32,
                ..Default::default()
            };

            match client.search(req).await {
                Ok(resp) => {
                    model.update(&mut cx, |this, cx| {
                        let rows = resp.hits.into_iter().map(|h| ResultRow {
                            doc_key: h.key,
                            name: h.name.unwrap_or_default(),
                            path: h.path.unwrap_or_default(),
                            ext: h.ext.unwrap_or_default(),
                            size: h.size.unwrap_or_default(),
                            modified_ts: h.modified.unwrap_or_default(),
                            score: h.score,
                        }).collect();
                        
                        this.update_results(rows, resp.total as usize, resp.took_ms as u64, cx);
                        this.set_connected(true, cx);
                    }).ok();
                }
                Err(e) => {
                    tracing::error!("Search failed: {e}");
                    model.update(&mut cx, |this, cx| {
                        this.set_connected(false, cx);
                    }).ok();
                }
            }
        }))
    }

    pub fn update_results(
        &mut self, 
        rows: Vec<ResultRow>, 
        total: usize, 
        latency_ms: u64, 
        cx: &mut ModelContext<Self>
    ) {
        let truncated = rows.len() < total;
        self.status.last_latency_ms = Some(latency_ms);
        self.status.shown = rows.len();
        self.status.total = total;
        self.status.truncated = truncated;
        self.results = rows;
        // Reset selection if invalid
        if let Some(idx) = self.selected_index {
            if idx >= self.results.len() {
                self.selected_index = None;
            }
        }
        cx.notify();
    }

    pub fn set_connected(&mut self, connected: bool, cx: &mut ModelContext<Self>) {
        if self.status.connected != connected {
            self.status.connected = connected;
            cx.notify();
        }
    }

    pub fn select_index(&mut self, index: Option<usize>, cx: &mut ModelContext<Self>) {
        self.selected_index = index;
        cx.notify();
    }
    
    pub fn selected_row(&self) -> Option<&ResultRow> {
        self.selected_index.and_then(|i| self.results.get(i))
    }
}