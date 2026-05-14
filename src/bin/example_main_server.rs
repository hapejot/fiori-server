use axum::{
    body::Body,
    response::Response,
    routing::{get, post},
    Router,
};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

use simple_fiori_server::{app_state::AppState, entity::ODataEntity, spec, BASE_PATH};
use simple_fiori_server::{entity::ODataEntityImp, runtime::handlers::*, NAMESPACE};
use simple_fiori_server::settings::Settings;

fn webapp_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_default().join("webapp")
}
use serde_json::{json, Value};

use crate::spec::EntitySpec;

#[derive(Debug)]
pub struct ExampleOrderEntity;

#[derive(Debug)]
pub struct ExampleOrderItemEntity;
impl ExampleOrderItemEntity {
    fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct ExampleCustomerEntity;

use crate::spec::{FacetSectionSpec, FieldSpec, Relationship, Side};

/// Central generator for a minimal example service model.
///
/// Contains three entities: Orders (parent), OrderItems (composition child),
/// and Customers (lookup/master).
pub fn entities() -> Vec<EntitySpec> {
    vec![order_spec(), order_item_spec(), customer_spec()]
}

/// Relationships connecting the example entities.
pub fn relationships() -> Vec<Relationship> {
    vec![
        Relationship {
            name: "Order_Items".into(),
            one: Side::new("Orders", "Items"),
            many: Side::new("OrderItems", "_Order"),
            owned: true,
            fk_field: Some("OrderID".into()),
            fk_label: Some("Order".into()),
            fk_form_group: Some("General".into()),
            condition: None,
            package: None,
        },
        Relationship {
            name: "Order_Customer".into(),
            one: Side::new("Customers", "Orders"),
            many: Side::new("Orders", "Customer"),
            owned: false,
            fk_field: Some("CustomerID".into()),
            fk_label: Some("Customer".into()),
            fk_form_group: Some("General".into()),
            condition: None,
            package: None,
        },
    ]
}

pub fn order_spec() -> EntitySpec {
    EntitySpec {
        set_name: "Orders".into(),
        package: None,
        type_name: Some("Order".into()),
        type_name_plural: Some("Orders".into()),
        title_field: Some("OrderName".into()),
        description_field: None,
        fields: vec![
            FieldSpec::string("OrderName", "Order Name", 80)
                .searchable()
                .form_group("FGQ")
                .show_in_list(),
            FieldSpec::atom("OrderDate", "Order Date", "Edm.Date").show_in_list(),
            FieldSpec::decimal("TotalAmount", "Total Amount", 13, 2).show_in_list(),
        ],
        data_points: vec![],
        header_facets: vec![],
        facet_sections: vec![FacetSectionSpec {
            label: "General".into(),
            id: "orders-general".into(),
            field_group_qualifier: "FGQ".into(),
        }],
        table_facets: vec![],
    }
}

pub fn order_item_spec() -> EntitySpec {
    EntitySpec {
        set_name: "OrderItems".into(),
        package: None,
        type_name: Some("OrderItem".into()),
        type_name_plural: Some("Order Items".into()),
        title_field: Some("Description".into()),
        description_field: None,
        fields: vec![
            FieldSpec::string("Description", "Description", 120)
                .searchable()
                .show_in_list(),
            FieldSpec::int("Quantity", "Quantity").show_in_list(),
            FieldSpec::decimal("NetAmount", "Net Amount", 13, 2).show_in_list(),
        ],
        data_points: vec![],
        header_facets: vec![],
        facet_sections: vec![],
        table_facets: vec![],
    }
}

pub fn customer_spec() -> EntitySpec {
    EntitySpec {
        set_name: "Customers".into(),
        package: None,
        type_name: Some("Customer".into()),
        type_name_plural: Some("Customers".into()),
        title_field: Some("CustomerName".into()),
        description_field: None,
        fields: vec![
            FieldSpec::string("CustomerName", "Customer Name", 80)
                .searchable()
                .show_in_list(),
            FieldSpec::string("Email", "E-Mail", 120).show_in_list(),
        ],
        data_points: vec![],
        header_facets: vec![],
        facet_sections: vec![],
        table_facets: vec![],
    }
}

impl ODataEntityImp for ExampleOrderEntity {
    fn set_name(&self) -> &'static str {
        "Orders"
    }

    fn type_name(&self) -> &'static str {
        "Order"
    }

    fn initial_data(&self) -> Vec<Value> {
        vec![json!({
            "ID": "11111111-1111-1111-1111-111111111111",
            "OrderName": "Laptop Bundle",
            "OrderDate": "2026-05-01",
            "TotalAmount": 2499.00,
            "CustomerID": "33333333-3333-3333-3333-333333333333"
        })]
    }

    fn entity_spec(&self) -> Option<EntitySpec> {
        Some(order_spec())
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"Orders\" EntityType=\"{ns}.Order\"/>",
            ns = NAMESPACE
        )
    }
}

impl ODataEntityImp for ExampleOrderItemEntity {
    fn set_name(&self) -> &'static str {
        "OrderItems"
    }

    fn type_name(&self) -> &'static str {
        "OrderItem"
    }

    fn parent_set_name(&self) -> Option<&'static str> {
        Some("Orders")
    }

    fn initial_data(&self) -> Vec<Value> {
        vec![json!({
            "ID": "22222222-2222-2222-2222-222222222222",
            "Description": "15-inch Laptop",
            "Quantity": 1,
            "NetAmount": 2499.00,
            "OrderID": "11111111-1111-1111-1111-111111111111"
        })]
    }

    fn entity_spec(&self) -> Option<EntitySpec> {
        Some(order_item_spec())
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"OrderItems\" EntityType=\"{ns}.OrderItem\"/>",
            ns = NAMESPACE
        )
    }

    fn navigation_properties(&self) -> &'static [spec::NavigationPropertyDef] {
        &[]
    }
}

impl ODataEntityImp for ExampleCustomerEntity {
    fn set_name(&self) -> &'static str {
        "Customers"
    }

    fn type_name(&self) -> &'static str {
        "Customer"
    }

    fn initial_data(&self) -> Vec<Value> {
        vec![json!({
            "ID": "33333333-3333-3333-3333-333333333333",
            "CustomerName": "Acme Corp",
            "Email": "procurement@acme.example"
        })]
    }

    fn entity_spec(&self) -> Option<EntitySpec> {
        Some(customer_spec())
    }

    fn entity_set(&self) -> String {
        format!(
            "<EntitySet Name=\"Customers\" EntityType=\"{ns}.Customer\"/>",
            ns = NAMESPACE
        )
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let host = "0.0.0.0";
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8001);

    let settings = Settings::load(&webapp_dir().join("config/settings.json"));
    println!("{}", "=".repeat(60));
    println!("  Example Mode : Hardcoded Orders/OrderItems/Customers");
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
    println!("  Storage      : In-Memory");
    println!("{}", "=".repeat(60));
    println!("  Press Ctrl+C to stop\n");
    let order_item_entity = ExampleOrderItemEntity::new();
    let data_dir = std::env::current_dir().unwrap_or_default().join("data");
    let builder = AppState::builder()
        .settings(settings)
        .data_dir(&data_dir)
        .entity(ODataEntity::new(Arc::new(ExampleOrderEntity)))
        .entity(ODataEntity::new(Arc::new(order_item_entity)))
        .entity(ODataEntity::new(Arc::new(ExampleCustomerEntity)))
        .relationships(relationships());

    let app_state = Arc::new(builder.build());

    let base = BASE_PATH;
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
