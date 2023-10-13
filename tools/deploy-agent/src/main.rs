use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::post;
use axum::{Router, TypedHeader};
use axum_macros::debug_handler;
use clap::Parser;
use color_eyre::Result;
use reqwest::StatusCode;
use serde::Deserialize;
use tracing::error;

use crate::azure::Azure;
use crate::github::Signature;
use crate::webhook::handle_webhook;

mod azure;
mod github;
mod unzip;
mod webhook;

#[derive(Parser, Debug)]
pub struct ExecutableArgs {
    pub vault_name: String,
    pub extraction_directory: String,
}

pub struct AppState {
    pub azure: Azure,
    pub args: ExecutableArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = ExecutableArgs::parse();
    let azure = Azure::new(&args.vault_name)?;

    let state = Arc::new(AppState { azure, args });

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let app = Router::new()
        .route("/echo", get(echo))
        .route("/github", post(github_webhook))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 2374));
    Ok(axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?)
}

#[derive(Deserialize)]
struct EchoParams {
    message: String,
}

#[debug_handler]
async fn echo(params: Query<EchoParams>) -> impl IntoResponse {
    format!("Echo: {}", params.message)
}

#[debug_handler]
async fn github_webhook(
    TypedHeader(signature): TypedHeader<Signature>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    body: String,
) -> impl IntoResponse {
    match handle_webhook(&signature, &headers, &state, &body).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("Error when handling webhook {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
