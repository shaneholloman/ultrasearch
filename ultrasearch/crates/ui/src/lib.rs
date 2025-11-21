//! UI data model scaffolding for the GPUI client (c00.7.1/7.2).
//!
//! Keeps rendering light for now; focuses on state structures that other crates
//! can reuse without pulling in heavy UI wiring yet.

use core_types::DocKey;

const DEFAULT_MAX_ROWS: usize = 500;

#[derive(Debug, Clone)]
pub struct ResultRow {
    pub doc_key: DocKey,
    pub name: String,
    pub path: String,
    pub ext: String,
    pub size: u64,
    pub modified_ts: i64,
    pub score: f32,
}

#[derive(Debug, Default)]
pub struct ResultsStore {
    rows: Vec<ResultRow>,
    max_rows: usize,
}

impl ResultsStore {
    pub fn new(max_rows: usize) -> Self {
        Self {
            rows: Vec::new(),
            max_rows: max_rows.max(1),
        }
    }

    pub fn set_results(&mut self, mut rows: Vec<ResultRow>) {
        if rows.len() > self.max_rows {
            rows.truncate(self.max_rows);
        }
        self.rows = rows;
    }

    pub fn rows(&self) -> &[ResultRow] {
        &self.rows
    }

    pub fn clear(&mut self) {
        self.rows.clear();
    }
}

#[derive(Debug, Default)]
pub struct SearchStatus {
    pub last_latency_ms: Option<u64>,
    pub shown: usize,
    pub total: usize,
    pub truncated: bool,
    pub backend_mode: BackendMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendMode {
    #[default]
    MetadataOnly,
    Mixed,
    ContentOnly,
}

#[derive(Debug, Default)]
pub struct SearchAppModel {
    pub query: String,
    pub status: SearchStatus,
    results: ResultsStore,
}

impl SearchAppModel {
    pub fn new(max_rows: usize) -> Self {
        Self {
            query: String::new(),
            status: SearchStatus::default(),
            results: ResultsStore::new(max_rows),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_MAX_ROWS)
    }

    pub fn set_query(&mut self, text: impl Into<String>) {
        self.query = text.into();
    }

    pub fn update_results(&mut self, rows: Vec<ResultRow>, total: usize, latency_ms: u64) {
        let truncated = rows.len() < total;
        self.status.last_latency_ms = Some(latency_ms);
        self.status.shown = rows.len();
        self.status.total = total;
        self.status.truncated = truncated;
        self.results.set_results(rows);
    }

    pub fn results(&self) -> &[ResultRow] {
        self.results.rows()
    }
}
