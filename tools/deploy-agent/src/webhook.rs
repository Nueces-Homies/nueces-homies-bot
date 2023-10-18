use std::ffi::OsStr;

use axum::http::HeaderMap;
use color_eyre::eyre::eyre;
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Result;
use ring::hmac;
use tracing::{info, warn};

use crate::{
    azure::Azure,
    github::{Signature, WorkflowRunConclusion, WorkflowRunEvent},
    AppState,
};

pub async fn handle_webhook(
    signature: &Signature,
    headers: &HeaderMap,
    state: &AppState,
    body: &str,
) -> Result<()> {
    match validate_signature(&state.azure, &signature.digest, &body).await {
        Ok(true) => (),
        Ok(false) => return Err(eyre!("bad signature")),
        Err(e) => return Err(e),
    }

    let event_header = headers
        .get("x-github-event")
        .wrap_err("did not find x-github-event in headers")?;
    let event_type = event_header
        .to_str()
        .wrap_err("x-github-event header had unparseable value")?;

    if "workflow_run" != event_type {
        warn!("Got unhandled event {}", event_type);
        return Ok(());
    }

    let workflow_run_event = serde_json::from_str::<WorkflowRunEvent>(&body)?;

    let workflow_run = &workflow_run_event.workflow_run;
    if workflow_run.head_branch == "main"
        && workflow_run.path.file_name() == Some(OsStr::new("rust.yml"))
        && workflow_run
            .conclusion
            .as_ref()
            .is_some_and(|c| c == &WorkflowRunConclusion::Success)
    {
        let artifacts_url = &workflow_run.artifacts_url;
        info!(
            "Downloading artifacts for {} from {}",
            &workflow_run.head_sha[0..7],
            artifacts_url
        );
        state
            .download_queue_tx
            .send(artifacts_url.to_owned())
            .await?;
    }

    Ok(())
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
