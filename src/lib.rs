//! MiniERP — shared library.
//!
//! This crate is compiled for both the frontend (WASM) and server (native)
//! binaries. Platform-specific code is gated with `#[cfg(not(target_arch = "wasm32"))]`.
//!
//! # Modules
//!
//! | Module | Contents | Platform |
//! |--------|----------|----------|
//! | `models` | Shared data types (User, Invoice, Item, …) | Both |
//! | `api` | Frontend HTTP client trait + reqwest implementation | Both (wasm-compat) |
//! | `auth` | Auth context, login page, route guard | Both (frontend) |
//! | `server` | Axum routes, rusqlite DB, migrations | Native only |
//! | `components` | UI components (DataGrid, …) | Both (frontend) |
//! | `pages` | Page components | Both (frontend) |

#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod calculations;
pub mod models;
pub mod money;
pub mod api;
pub mod auth;
pub mod utils;
pub mod i18n;

// Server module — only compiled on native targets (not WASM)
#[cfg(not(target_arch = "wasm32"))]
pub mod server;

// UI modules — compiled everywhere but only instantiated by the frontend
pub mod components;
pub mod pages;
