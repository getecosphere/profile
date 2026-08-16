use mongodb::Database;
use std::sync::Arc;

use crate::{auth_client::AuthClient, config::AppConfig, storage_client::StorageClient};

#[derive(Clone)]
pub struct AppState(pub Arc<AppStateInner>);

pub struct AppStateInner {
    pub db: Database,
    pub config: AppConfig,
    pub auth_client: AuthClient,
    pub storage_client: StorageClient,
}

impl AppState {
    pub fn new(db: Database, config: AppConfig) -> Self {
        let auth_client = AuthClient::new(config.auth_base_url.clone());
        let storage_client = StorageClient::new(config.storage_base_url.clone());
        AppState(Arc::new(AppStateInner {
            db,
            config,
            auth_client,
            storage_client,
        }))
    }
}

impl std::ops::Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
