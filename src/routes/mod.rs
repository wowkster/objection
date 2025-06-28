use axum::Router;

use api::create_api_router;
use fs::create_fs_router;

use crate::AppState;

mod api;
mod fs;

pub fn create_router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/", create_fs_router(state.clone()))
        .nest("/api", create_api_router(state.clone()))
}
