//! MiniERP API Server — Binary Entry Point
//!
//! This binary is compiled only on native targets (not WASM).
//! It starts an Axum HTTP server with:
//! - rusqlite database (WAL mode, auto-migrated, auto-seeded)
//! - JWT authentication (login, logout, me)
//! - CORS enabled for the frontend
//! - Tracing/logging
//!
//! # Usage
//!
//! ```bash
//! # Build and run the server
//! cargo run --bin mini-erp-server
//!
//! # With custom port and DB path
//! MINI_ERP_PORT=8080 MINI_ERP_DB_PATH=/data/mini-erp.db cargo run --bin mini-erp-server
//! ```

// Allow dead code during early development
#![allow(dead_code)]

use mini_erp::server::routes;
use mini_erp::server::auth_routes::AppState;

/// Default port for the API server.
const DEFAULT_PORT: u16 = 3001;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Determine port from env var or default
    let port: u16 = std::env::var("MINI_ERP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    // Initialize the database (runs migrations and seed data)
    tracing::info!("Initializing database…");
    mini_erp::server::db::get_db(); // This triggers lazy initialization

    // Create app state and router
    let state = AppState::new();
    let app = routes::create_router(state);

    // Bind and serve
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("MiniERP API server starting on {}", addr);
    tracing::info!("Database: {}", std::env::var("MINI_ERP_DB_PATH").unwrap_or_else(|_| "./mini-erp.db".to_string()));

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", addr));

    tracing::info!("Server ready — listening on {}", addr);
    axum::serve(listener, app)
        .await
        .expect("Server error");
}
