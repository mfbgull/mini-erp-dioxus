use super::auth_routes::{self, AppState};
use super::*;
use axum::Router;
use axum::http::{HeaderName, HeaderValue};
use tower_http::cors::{AllowHeaders, AllowMethods, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:8080".parse::<HeaderValue>().unwrap(),
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:8080".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:3000".parse::<HeaderValue>().unwrap(),
            "http://localhost:3001".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:3001".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods(AllowMethods::any())
        .allow_headers(["content-type", "authorization", "accept"].map(|s| s.parse::<HeaderName>().unwrap()));

    Router::new()
        .merge(auth_routes::router())
        .merge(inventory_routes::router())
        .merge(customer_routes::router())
        .merge(invoice_routes::router())
        .merge(payment_routes::router())
        .merge(sales_routes::router())
        .merge(purchase_routes::router())
        .merge(manufacturing_routes::router())
        .merge(accounting_routes::router())
        .merge(admin_routes::router())
        .merge(dashboard_routes::router())
        .merge(report_routes::router())
        .merge(forecast_routes::router())
        .merge(pos_routes::router())
        .merge(mobile_routes::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
