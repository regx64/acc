use std::sync::Arc;

use object_store::ObjectStore;
use redis::aio::ConnectionManager;
use sqlx::PgPool;

use crate::config::Config;
use crate::mail::Mailer;

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<Config>,
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub store: Arc<dyn ObjectStore>,
    pub mail: Mailer,
    pub http: reqwest::Client,
}

impl AppState {
    /// Best-effort operator alert (Discord-compatible webhook).
    pub async fn alert(&self, text: &str) {
        let Some(url) = &self.cfg.alert_webhook else {
            return;
        };
        let text: String = text.chars().take(1800).collect();
        let _ = self
            .http
            .post(url)
            .json(&serde_json::json!({ "content": format!("[acc] {text}") }))
            .send()
            .await;
    }
}
