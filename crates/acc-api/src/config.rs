use anyhow::Context;

#[derive(Clone)]
pub struct Config {
    pub bind: String,
    pub database_url: String,
    pub redis_url: String,
    pub storage_url: String,
    /// Public URL of the web frontend, used in mail links.
    pub web_url: String,
    /// Shared secret the SvelteKit server sends so its forwarded client IP is trusted.
    pub internal_token: Option<String>,
    /// Trust `CF-Connecting-IP` (API reachable only through Cloudflare).
    pub trust_cloudflare: bool,
    /// `Secure` flag on the session cookie.
    pub secure_cookies: bool,
    pub resend_api_key: Option<String>,
    pub mail_from: String,
    /// Discord-compatible webhook for operator alerts (typo reports).
    pub alert_webhook: Option<String>,
    /// Submissions are refused while the queue is longer than this.
    pub queue_limit: usize,
    /// Handles that get the ADMIN role on signup.
    pub admin_handles: Vec<String>,
}

fn var(name: &str) -> anyhow::Result<String> {
    std::env::var(name).with_context(|| format!("{name} is not set"))
}

fn opt(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            bind: opt("BIND").unwrap_or_else(|| "0.0.0.0:8080".into()),
            database_url: var("DATABASE_URL")?,
            redis_url: var("REDIS_URL")?,
            storage_url: var("STORAGE_URL")?,
            web_url: opt("WEB_URL").unwrap_or_else(|| "http://localhost:5173".into()),
            internal_token: opt("INTERNAL_TOKEN"),
            trust_cloudflare: opt("TRUST_CLOUDFLARE").is_some_and(|v| v == "1" || v == "true"),
            secure_cookies: opt("SECURE_COOKIES").is_none_or(|v| v != "0" && v != "false"),
            resend_api_key: opt("RESEND_API_KEY"),
            mail_from: opt("MAIL_FROM").unwrap_or_else(|| "acc <no-reply@localhost>".into()),
            alert_webhook: opt("ALERT_WEBHOOK_URL"),
            queue_limit: opt("QUEUE_LIMIT")
                .and_then(|v| v.parse().ok())
                .unwrap_or(500),
            admin_handles: opt("ADMIN_HANDLES")
                .map(|v| {
                    v.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_else(|| vec!["regx64".into()]),
        })
    }
}
