use anyhow::Result;
use ipc::{SearchRequest, SearchResponse};
use std::path::Path;
use std::sync::OnceLock;

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

/// Placeholder handler that will eventually query the metadata index.
#[derive(Debug, Default)]
pub struct MetaIndexSearchHandler;

impl MetaIndexSearchHandler {
    pub fn try_new(_index_path: &Path) -> Result<Self> {
        // TODO: wire Tantivy reader here.
        Ok(Self)
    }
}

impl SearchHandler for MetaIndexSearchHandler {
    fn search(&self, req: SearchRequest) -> SearchResponse {
        StubSearchHandler.search(req)
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
