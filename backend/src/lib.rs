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
pub mod request_id;
pub mod routes;
pub mod state;

pub async fn bootstrap() -> anyhow::Result<axum::Router> {
    let _ = dotenvy::dotenv();
    let config = config::AppConfig::from_env()?;
    let client = mongodb::Client::with_uri_str(&config.mongodb_uri).await?;
    let db = client.default_database()
        .ok_or_else(|| anyhow::anyhow!("MONGODB_URI must include a database name"))?;
    let state = state::AppState::new(db, config.clone());
    Ok(routes::build_router(state))
}
