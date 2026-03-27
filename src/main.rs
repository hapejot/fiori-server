mod annotations;
mod app_state;
mod builders;
mod draft;
mod entities;
mod entity;
mod handlers;
mod query;
mod routing;
mod settings;

use axum::{
    routing::{get, post},
    Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use app_state::AppState;
use entities::{OrderEntity, OrderItemEntity, ProductEntity};
use handlers::*;
use settings::Settings;

pub const BASE_PATH: &str = "/odata/v4/ProductsService";
pub const NAMESPACE: &str = "ProductsService";

// ── Webapp directory (sibling to the executable's working dir) ──────────
pub fn webapp_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_default().join("webapp")
}

#[tokio::main]
async fn main() {
    // Logger initialisieren (RUST_LOG=info fuer Standard, =debug fuer mehr)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let host = "0.0.0.0";
    let port = 8000u16;

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
        "  manifest     : http://localhost:{}/manifest.json (dynamisch)",
        port
    );
    println!(
        "  Products     : http://localhost:{}{}/Products",
        port, BASE_PATH
    );
    println!(
        "  Single Item  : http://localhost:{}{}/Products('P001')",
        port, BASE_PATH
    );
    println!("{}", "=".repeat(60));
    println!("  Druecke Ctrl+C zum Beenden\n");

    let data_dir = std::env::current_dir().unwrap_or_default().join("data");
    let app_state = Arc::new(
        AppState::builder()
            .settings(settings)
            .data_dir(&data_dir)
            .entity(&ProductEntity)
            .entity(&OrderEntity)
            .entity(&OrderItemEntity)
            .build(),
    );

    let base = BASE_PATH;

    // Routen fuer jedes registrierte EntitySet dynamisch erzeugen
    let mut entity_routes = Router::new();
    for entity in app_state.entities.iter() {
        let set = entity.set_name();
        entity_routes = entity_routes
            .route(
                &format!("{}/{}", base, set),
                get(collection_handler).head(collection_handler),
            )
            .route(
                &format!("{}/{}/$count", base, set),
                get(count_handler),
            );
    }

    let app = Router::new()
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
    axum::serve(listener, app).await.unwrap();
}
