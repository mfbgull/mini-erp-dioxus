use super::auth_routes::{self, AppState};
use super::*;
use axum::Router;
use axum::http::{header, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS, Method::PATCH])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
        ]);

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
