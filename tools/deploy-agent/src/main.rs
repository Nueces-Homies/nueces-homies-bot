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
use github::{WorkflowRunEvent, WorkflowRunConclusion};
use reqwest::StatusCode;
use ring::hmac;
use tracing::{error, warn, info};

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
        .with_max_level(tracing::Level::INFO)
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
            error!("Encountered error {}", e);
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

    if "workflow_run" != event_type {
        warn!("Got unhandled event {}", &event_type);
        return StatusCode::OK;
    }

    let workflow_run_event = match serde_json::from_str::<WorkflowRunEvent>(&body) {
        Ok(event) => event,
        Err(error) => {
            error!("Unable to deserialize body {}. Error {}", &body, error);
            return StatusCode::BAD_REQUEST;
        }
    };

    let workflow_run = &workflow_run_event.workflow_run;
    if workflow_run.head_branch == "main"  && workflow_run.conclusion.as_ref().is_some_and(|c| c == &WorkflowRunConclusion::Success) {
        let artifacts_url = &workflow_run.artifacts_url;
        info!("Downloading artifacts for {} from {}", &workflow_run.head_sha[0..7], artifacts_url);
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
