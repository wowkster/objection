use std::{net::SocketAddr, path::Path, sync::Arc};

use crate::{config::Config, routes::create_router};
use axum::{
    Json, Router, ServiceExt,
    extract::{FromRef, Request},
    http::{HeaderValue, StatusCode, Uri},
};
use serde_json::{Value, json};
use sqlx::migrate::MigrateDatabase;
use tower_http::{cors::CorsLayer, normalize_path::NormalizePath, trace::TraceLayer};

use tokio::task::JoinHandle;

pub mod config;
mod models;
mod routes;

#[derive(Clone, FromRef)]
struct AppState {
    db: sqlx::SqlitePool,
    config: Arc<Config>,
}

pub async fn create_server(config: Config) -> (SocketAddr, JoinHandle<()>) {
    /* Initialize State */

    init_data_directory(&config.data_directory).expect("Failed to create data directory");

    let db = init_main_db(&config.data_directory)
        .await
        .expect("Failed to initialize DB");

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

    let (host, port) = (config.http.host, config.http.port);
    let state = AppState {
        db,
        config: Arc::new(config),
    };

    let app = NormalizePath::trim_trailing_slash(
        Router::new()
            .fallback(fallback)
            .merge(create_router(state.clone()))
            .layer(cors)
            .layer(TraceLayer::new_for_http())
            .with_state(state),
    );

    /* Serve our app with hyper */

    let addr = SocketAddr::from((host, port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    tracing::info!("Listening on: http://{}", addr);

    (
        listener.local_addr().unwrap(),
        tokio::spawn(async {
            axum::serve(
                listener,
                ServiceExt::<Request>::into_make_service_with_connect_info::<SocketAddr>(app),
            )
            .await
            .unwrap()
        }),
    )
}

fn init_data_directory(path: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(path.as_ref())?;

    // TODO: create folder structure?

    Ok(())
}

async fn init_main_db(data_directory: impl AsRef<Path>) -> sqlx::Result<sqlx::SqlitePool> {
    let database_url = format!(
        "sqlite://{}",
        data_directory.as_ref().join("database.sqlite3").display()
    );

    if !sqlx::Sqlite::database_exists(&database_url)
        .await
        .unwrap_or(false)
    {
        tracing::debug!("Creating new database at {}", database_url);
        sqlx::Sqlite::create_database(&database_url).await?;
    } else {
        tracing::debug!("Database already exists at {}", database_url);
    }

    let db = sqlx::SqlitePool::connect(&database_url).await?;

    sqlx::migrate!("migrations/instance").run(&db).await?;

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
