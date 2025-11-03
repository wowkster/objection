use axum::{Json, Router, extract::Request, routing::get};
use buckets::create_buckets_router;
use serde::Deserialize;
use serde_json::json;

use crate::AppState;

mod buckets;

pub fn create_api_router(state: AppState) -> Router<AppState> {
    Router::new().nest("/buckets", create_buckets_router())
}

#[derive(Debug, Deserialize)]
struct PaginatedQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
