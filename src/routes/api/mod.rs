use axum::{routing::get, Json, Router};
use buckets::create_buckets_router;
use serde::Deserialize;
use serde_json::json;

use crate::AppState;

mod buckets;

pub fn create_api_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(|| async {
                Json(json!({
                    "message": "Welcome to the Objection API!",
                    "version": env!("CARGO_PKG_VERSION")
                }))
            }),
        )
        .nest("/buckets", create_buckets_router())
}

#[derive(Debug, Deserialize)]
struct PaginatedQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}
