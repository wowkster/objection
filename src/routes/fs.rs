use axum::Router;

use crate::AppState;

pub fn create_fs_router(state: AppState) -> Router<AppState> {
    Router::new()
}
