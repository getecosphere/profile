//! Library surface so `tests/` (real integration tests) can build a real
//! `Router` and `AppState` against a real MongoDB, exactly like `main.rs`
//! does. `main.rs` is a thin wrapper around this crate.
pub mod auth_client;
pub mod auth_extractor;
pub mod config;
pub mod date_parse;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod jwt;
pub mod models;
pub mod repo;
pub mod routes;
pub mod state;
