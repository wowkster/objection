use axum::{Router, extract::Request, routing::get};

use api::create_api_router;

use crate::AppState;

mod api;

pub fn create_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(handle_index))
        .nest("/api", create_api_router(state.clone()))
}

async fn handle_index(req: Request) {
    eprintln!("req = {req:?}");
    todo!()
}
