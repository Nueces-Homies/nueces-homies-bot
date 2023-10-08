use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::extract::State;
use axum::headers::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Router, TypedHeader};
use axum_macros::debug_handler;
use clap::Parser;
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventPayload};
use ring::hmac;

use crate::azure::Azure;
use crate::github::{download_and_extract_github_artifact, Signature};

mod azure;
mod github;
mod unzip;

#[derive(Parser, Debug)]
struct ExecutableArgs {
    vault_name: String,
    extraction_directory: String,
}

struct AppState {
    azure: Azure,
    args: ExecutableArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = ExecutableArgs::parse();
    let azure = Azure::new(&args.vault_name)?;

    let state = Arc::new(AppState { azure, args });

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/github", post(github_webhook))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 2374));
    Ok(axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?)
}

#[debug_handler]
async fn github_webhook(
    TypedHeader(signature): TypedHeader<Signature>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    body: String,
) -> impl IntoResponse {
    let secret = state
        .azure
        .get_secret("github-webhook-secret")
        .await
        .unwrap();

    let digest = signature.digest;
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());

    if let Err(_) = hmac::verify(&key, body.as_bytes(), digest.as_slice()) {
        return StatusCode::BAD_REQUEST;
    }

    let event_type = headers.get("x-github-event").unwrap().to_str().unwrap();
    let event = WebhookEvent::try_from_header_and_body(event_type, &body).unwrap();
    if let WebhookEventPayload::WorkflowRun(workflow_event) = event.specific {
        let run_info = workflow_event.workflow_run;
        let status = &run_info["status"];
        let branch = &run_info["head_branch"];

        if branch == "main" && status == "completed" && run_info["conclusion"] == "success" {
            let artifacts_url = &run_info["artifacts_url"];
            download_and_extract_github_artifact(
                &state.azure,
                &artifacts_url.to_string(),
                &state.args.extraction_directory,
            )
            .await
            .unwrap();
        }
    }

    StatusCode::OK
}
