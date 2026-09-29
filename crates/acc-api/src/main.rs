//! acc HTTP API.
//!
//! `acc-api --openapi` prints the OpenAPI document and exits (used to
//! generate the frontend's TypeScript types).

mod auth;
mod config;
mod error;
mod mail;
mod page;
mod ratelimit;
mod routes;
mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::routing::get;
use axum::{Json, Router};
use sqlx::postgres::PgPoolOptions;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::config::Config;
use crate::mail::Mailer;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--openapi") {
        println!("{}", routes::api().into_openapi().to_pretty_json()?);
        return Ok(());
    }
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let cfg = Config::from_env()?;
    let db = PgPoolOptions::new()
        .max_connections(16)
        .connect(&cfg.database_url)
        .await
        .context("connecting to postgres")?;
    sqlx::migrate!("../../migrations")
        .run(&db)
        .await
        .context("running migrations")?;
    let redis = redis::Client::open(cfg.redis_url.as_str())?
        .get_connection_manager()
        .await
        .context("connecting to redis")?;
    let store = acc_core::storage::from_url(&cfg.storage_url)?;
    let http = reqwest::Client::new();
    let mail = Mailer::new(
        http.clone(),
        cfg.resend_api_key.clone(),
        cfg.mail_from.clone(),
    );
    let bind = cfg.bind.clone();
    let state = AppState {
        cfg: Arc::new(cfg),
        db,
        redis,
        store,
        mail,
        http,
    };

    let app = app(state);
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    info!("listening on {bind}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;
    Ok(())
}

fn app(state: AppState) -> Router {
    let (api, openapi) = routes::api().split_for_parts();
    let openapi = Arc::new(openapi);
    let api = api.route(
        "/openapi.json",
        get(move || {
            let doc = openapi.clone();
            async move { Json((*doc).clone()) }
        }),
    );

    // Coarse per-IP flood guard; the plan's exact limits live in `ratelimit`.
    let governor = GovernorConfigBuilder::default()
        .per_millisecond(100)
        .burst_size(60)
        .key_extractor(auth::ClientIpKey)
        .finish()
        .expect("governor config");

    Router::new()
        .nest("/api/v1", api)
        .layer(GovernorLayer::new(governor))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::client_ip_layer,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
