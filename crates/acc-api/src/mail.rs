//! Outgoing mail through Resend. Without an API key, mail is logged instead.

use serde_json::json;

#[derive(Clone)]
pub struct Mailer {
    http: reqwest::Client,
    api_key: Option<String>,
    from: String,
}

impl Mailer {
    pub fn new(http: reqwest::Client, api_key: Option<String>, from: String) -> Self {
        Mailer {
            http,
            api_key,
            from,
        }
    }

    pub async fn send(&self, to: &str, subject: &str, text: &str) {
        let Some(key) = &self.api_key else {
            tracing::info!(%to, %subject, "mail (not sent, no RESEND_API_KEY):\n{text}");
            return;
        };
        let res = self
            .http
            .post("https://api.resend.com/emails")
            .bearer_auth(key)
            .json(&json!({ "from": self.from, "to": [to], "subject": subject, "text": text }))
            .send()
            .await;
        match res {
            Ok(r) if r.status().is_success() => {}
            Ok(r) => tracing::error!(status = %r.status(), "resend rejected mail"),
            Err(e) => tracing::error!("resend: {e}"),
        }
    }
}
