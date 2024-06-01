use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::{broadcast, RwLock};

use super::chat::Msg;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub tx: broadcast::Sender<Arc<RwLock<Msg>>>,
}

impl AppState {
    pub fn new(db: SqlitePool) -> Self {
        let (tx, _) = broadcast::channel(64);
        Self { db, tx }
    }
}
