//! # simple Fiori Server
//! 
//! OData V4 server for SAP Fiori Elements...
//! 
//! Main modules are
//! - `spec` high level definition of relationships and entities
//! - `entities` reconstructs entity configs from data directory and creates generic entities
//! - `runtime` contains the actual web server and request handlers
//! - `main` initializes everything and starts the server


use axum::{
    body::Body,
    response::Response,
    routing::{get, post},
    Router,
};
use tracing::info;

use std::path::PathBuf;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use simple_fiori_server::{app_state::AppState, entity::ODataEntity, *};
use entities::{
    EntityConfigEntity, EntityFacetEntity, EntityFieldEntity, EntityNavigationEntity,
    EntityRelationshipEntity, EntityTableFacetEntity, FieldValueListEntity,
    FieldValueListItemEntity,
};
use runtime::handlers::*;
use simple_fiori_server::settings::Settings;


// ── Webapp directory (sibling to the executable's working dir) ──────────
pub fn webapp_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_default().join("webapp")
}

#[tokio::main]
async fn main() {
    // Initialize logger (RUST_LOG=info for standard, =debug for more)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Load .env file if the postgres feature is enabled
    #[cfg(feature = "postgres")]
    {
        if let Err(e) = dotenvy::dotenv() {
            tracing::debug!("No .env file loaded: {}", e);
        }
    }

    let host = "0.0.0.0";
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8000);

    let settings = Settings::load(&webapp_dir().join("config/settings.json"));
    println!("{}", "=".repeat(60));
    println!("  UI5 Version  : {}", settings.ui5_version);
    println!("  Theme        : {}", settings.theme);
    println!("  Language     : {}", settings.language);
    println!("{}", "=".repeat(60));
    println!("  Web App      : http://localhost:{}/", port);
    println!("  Service Root : http://localhost:{}{}", port, BASE_PATH);
    println!(
        "  $metadata    : http://localhost:{}{}/$metadata",
        port, BASE_PATH
    );
    println!(
        "  manifest     : http://localhost:{}/manifest.json (dynamic)",
        port
    );
    println!("{}", "=".repeat(60));

    let data_dir = std::env::current_dir().unwrap_or_default().join("data");

    // Meta tables in the data directory are the single source of truth.
    // Reconstruct EntityConfigs and create generic entities from them.
    let raw_configs = entities::meta::reconstruct_configs_from_data(&data_dir);

    let (generic_entities, generic_relationships) =
        entities::generic::create_generic_entities(raw_configs);
    info!(
        "Loaded {} generic entities and {} relationships from data directory",
        generic_entities.len(),
        generic_relationships.len()
    );
    for r in generic_relationships.iter() {
        info!("  {}: 1:{:?} -> N:{:?}", r.name, r.one, r.many);
    }

    let mut builder = AppState::builder()
        .settings(settings)
        .data_dir(&data_dir)
        .relationships(spec::meta_package::meta_relationships())
        .entity(ODataEntity::new(Arc::new(EntityConfigEntity)))
        .entity(ODataEntity::new(Arc::new(EntityFieldEntity)))
        .entity(ODataEntity::new(Arc::new(EntityFacetEntity)))
        .entity(ODataEntity::new(Arc::new(EntityNavigationEntity)))
        .entity(ODataEntity::new(Arc::new(EntityRelationshipEntity)))
        .entity(ODataEntity::new(Arc::new(EntityTableFacetEntity)))
        .entity(ODataEntity::new(Arc::new(FieldValueListEntity)))
        .entity(ODataEntity::new(Arc::new(FieldValueListItemEntity)))
        .relationships(generic_relationships);

    for ge in generic_entities {
        builder = builder.entity(ge);
    }

    // Select storage backend: PostgreSQL if DATABASE_URL is set, else in-memory
    #[cfg(feature = "postgres")]
    {
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            println!("  Storage      : PostgreSQL");
            let entities_for_pg = builder.entities.clone();
            match pg_store::PgDataStore::new(&database_url, data_dir.clone(), entities_for_pg).await
            {
                Ok(pg) => {
                    builder = builder.data_store(Box::new(pg));
                }
                Err(e) => {
                    eprintln!(
                        "  ERROR: Failed to connect to PostgreSQL: {}\n  Falling back to in-memory storage.",
                        e
                    );
                    println!("  Storage      : In-Memory (fallback)");
                }
            }
        } else {
            println!("  Storage      : In-Memory");
        }
    }
    #[cfg(not(feature = "postgres"))]
    {
        println!("  Storage      : In-Memory");
    }
    println!("{}", "=".repeat(60));
    println!("  Press Ctrl+C to stop\n");

    let app_state = Arc::new(builder.build());

    let base = BASE_PATH;

    // Dynamically create routes for each registered EntitySet
    let mut entity_routes = Router::new();

    for entity in app_state.entities.read().unwrap().iter() {
        info!("Registering EntitySet: {}", entity.set_name());
        let set = entity.set_name();
        entity_routes = entity_routes
            .route(
                &format!("{}/{}", base, set),
                get(collection_handler).head(collection_handler),
            )
            .route(&format!("{}/{}/$count", base, set), get(count_handler));
    }

    let app = Router::new()
        .route("/health", get(health_handler))
        .route(
            &format!("{}/$metadata", base),
            get(metadata_handler).head(metadata_handler),
        )
        .route(
            &format!("{}/", base),
            get(service_document).head(service_document),
        )
        .route(base, get(service_document).head(service_document))
        .route(&format!("{}/$batch", base), post(batch_handler))
        .merge(entity_routes)
        .fallback(catch_all)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind((host, port)).await.unwrap();

    // Graceful shutdown on SIGINT / SIGTERM
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn health_handler() -> Response {
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"status":"ok"}"#))
        .unwrap()
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { println!("\nReceived SIGINT, shutting down..."); },
        _ = terminate => { println!("\nReceived SIGTERM, shutting down..."); },
    }
}
