use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use axum::{
    extract::{FromRef, Request},
    http::{HeaderValue, StatusCode, Uri},
    Json, Router, ServiceExt,
};
use clap::Parser;
use config::Config;
use serde_json::{json, Value};
use tower_http::{cors::CorsLayer, normalize_path::NormalizePath, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

use crate::routes::create_router;

mod config;
// mod error;
// mod extractors;
// mod headers;
// mod middleware;
mod models;
mod routes;

#[derive(Debug, clap::Parser)]
pub struct Args {
    config_path: Option<PathBuf>,
}

type Database = Arc<sqlite::ConnectionThreadSafe>;

#[derive(Clone)]
struct AppState {
    database: Database,
}

impl FromRef<AppState> for Database {
    fn from_ref(app_state: &AppState) -> Database {
        app_state.database.clone()
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    /* Initialize Config */

    let args = Args::parse();

    let config = CONFIG.get_or_init(|| {
        args.config_path
            .map(Config::parse_and_validate)
            .unwrap_or_default()
    });

    tracing::debug!("using config: {:#?}", config);

    /* Initialize State */

    init_data_directory().expect("Failed to create data directory");

    let database = init_db().await.expect("Failed to initialize DB");

    let state = AppState { database };

    /* CORS Support */

    let cors = match &config.cors {
        Some(cors) => {
            let origins = cors
                .allow_origins
                .iter()
                .map(|o| o.ascii_serialization().parse().unwrap())
                .collect::<Vec<HeaderValue>>();

            CorsLayer::new()
                .allow_methods(cors.allow_methods.clone().into_iter().collect::<Vec<_>>())
                .allow_headers(cors.allow_headers.clone().into_iter().collect::<Vec<_>>())
                .allow_credentials(cors.allow_credentials)
                .allow_origin(origins)
        }
        None => CorsLayer::new(),
    };

    /* Initialize Application */

    let app = NormalizePath::trim_trailing_slash(
        Router::new()
            .fallback(fallback)
            .merge(create_router(state.clone()))
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .with_state(state),
    );

    /* Serve our app with hyper */

    let addr = SocketAddr::from((config.http.host, config.http.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Listening on: http://{}", addr);

    axum::serve(
        listener,
        ServiceExt::<Request>::into_make_service_with_connect_info::<SocketAddr>(app),
    )
    .await
}

fn init_data_directory() -> std::io::Result<()> {
    std::fs::create_dir_all("data")
}

async fn init_db() -> sqlite::Result<Database> {
    let db = Arc::new(sqlite::Connection::open_thread_safe(
        Path::new("data").join("database.sqlite3"),
    )?);

    models::create_tables(&db).await?;

    tracing::debug!("Initialized database!");

    Ok(db)
}

/// Generic 404 fallback handler
async fn fallback(uri: Uri) -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "NOT_FOUND",
            "message": format!("The requested resource `{}` was not found", uri)
        })),
    )
}
