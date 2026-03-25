mod annotations;
mod app_state;
mod builders;
mod entities;
mod entity;
mod handlers;
mod query;
mod settings;

use actix_web::{web, App, HttpServer};
use std::path::PathBuf;

use app_state::AppState;
use entities::{OrderEntity, ProductEntity};
use handlers::*;
use settings::Settings;

pub const BASE_PATH: &str = "/odata/v4/ProductsService";
pub const NAMESPACE: &str = "ProductsService";

// ── Webapp directory (sibling to the executable's working dir) ──────────
pub fn webapp_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_default().join("webapp")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    let app_state = web::Data::new(
        AppState::builder()
            .settings(settings)
            .entity(&ProductEntity)
            .entity(&OrderEntity)
            .build(),
    );

    HttpServer::new(move || {
        let base = BASE_PATH;

        let mut app = App::new()
            .app_data(app_state.clone())
            .route(
                &format!("{}/$metadata", base),
                web::get().to(metadata_handler),
            )
            .route(
                &format!("{}/$metadata", base),
                web::head().to(metadata_handler),
            )
            .route(
                &format!("{}/$metadata", base),
                web::method(actix_web::http::Method::OPTIONS).to(options_handler),
            )
            .route(&format!("{}/", base), web::get().to(service_document))
            .route(&format!("{}/", base), web::head().to(service_document))
            .route(base, web::get().to(service_document))
            .route(base, web::head().to(service_document))
            .route(&format!("{}/$batch", base), web::post().to(batch_handler))
            .route(
                &format!("{}/$batch", base),
                web::method(actix_web::http::Method::OPTIONS).to(options_handler),
            );

        // Routen fuer jedes registrierte EntitySet dynamisch erzeugen
        for entity in app_state.entities.iter() {
            let set = entity.set_name();
            app = app
                .route(
                    &format!("{}/{}", base, set),
                    web::get().to(collection_handler),
                )
                .route(
                    &format!("{}/{}", base, set),
                    web::head().to(collection_handler),
                )
                .route(
                    &format!("{}/{}/$count", base, set),
                    web::get().to(count_handler),
                );
        }

        app.default_service(web::route().to(catch_all))
    })
    .bind((host, port))?
    .run()
    .await
}
