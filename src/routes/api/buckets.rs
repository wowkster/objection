use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::{
    AppState,
    models::bucket::{Bucket, BucketSettings},
};

pub fn create_buckets_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_buckets).post(post_buckets))
        .route(
            "/{name}",
            get(get_bucket).patch(patch_bucket).delete(delete_bucket),
        )
    // .route("/:name/objects", get(get_objects).post(handler))
}

#[derive(Debug, Serialize)]
struct ClientBucket {
    uuid: String,
    name: String,
    settings: BucketSettings,
}

impl From<Bucket> for ClientBucket {
    fn from(value: Bucket) -> Self {
        ClientBucket {
            uuid: value.uuid().to_string(),
            name: value.name().to_string(),
            settings: value.settings().clone(),
        }
    }
}

async fn get_buckets(State(db): State<sqlx::SqlitePool>) -> Json<Vec<ClientBucket>> {
    Json(
        Bucket::find_all(&db)
            .await
            .unwrap()
            .into_iter()
            .map(Into::into)
            .collect(),
    )
}

async fn post_buckets() {
    todo!()
}

async fn get_bucket() {
    todo!()
}

async fn patch_bucket() {
    todo!()
}

async fn delete_bucket() {
    todo!()
}
