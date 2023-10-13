use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Router, TypedHeader};
use axum_macros::debug_handler;
use clap::Parser;
use color_eyre::Result;
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventPayload};
use reqwest::StatusCode;
use ring::hmac;
use tracing::error;

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
    let digest = &signature.digest;

    match validate_signature(&state.azure, digest, &body).await {
        Ok(false) => {
            error!("Bad signature {:?}", digest);
            return StatusCode::BAD_REQUEST;
        }
        Err(e) => {
            error!("Encountered error {:?}", e);
            return StatusCode::BAD_REQUEST;
        }
        _ => (),
    }

    let event_header = match headers.get("x-github-event") {
        Some(header) => header,
        None => {
            error!("x-github-event not found in headears");
            return StatusCode::BAD_REQUEST;
        }
    };

    let event_type = match event_header.to_str() {
        Ok(value) => value,
        Err(e) => {
            error!("Got invalid header value for x-github-event {}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    let event = match WebhookEvent::try_from_header_and_body(event_type, &body) {
        Ok(event) => event,
        Err(e) => {
            error!("Unable to parse event type {}: {}", event_type, e);
            return StatusCode::BAD_REQUEST;
        }
    };

    if let WebhookEventPayload::WorkflowRun(workflow_event) = event.specific {
        let run_info = workflow_event.workflow_run;
        let status = &run_info["status"];
        let branch = &run_info["head_branch"];

        if branch == "main" && status == "completed" && run_info["conclusion"] == "success" {
            let artifacts_url = &run_info["artifacts_url"];
            if let Err(e) = download_and_extract_github_artifact(
                &state.azure,
                &artifacts_url.to_string(),
                &state.args.extraction_directory,
            )
            .await
            {
                error!(
                    "Failed to download and extract artifact {} from Github: {}",
                    &artifacts_url.to_string(),
                    e
                );
            }
        }
    }

    StatusCode::OK
}

async fn validate_signature(
    azure: &Azure,
    expected_signature: &Vec<u8>,
    data: &str,
) -> Result<bool> {
    let secret = azure.get_secret("github-webhook-secret").await?;
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    let result = hmac::verify(&key, data.as_bytes(), expected_signature.as_slice());
    Ok(result.is_ok())
}
