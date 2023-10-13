use std::io::Write;
use std::path::PathBuf;

use axum::headers;
use axum::headers::{Error as HeaderError, Header, HeaderName, HeaderValue};
use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use reqwest::header;
use serde::Deserialize;
use tempfile::NamedTempFile;

use crate::azure::Azure;
use crate::unzip::unzip;

#[derive(Deserialize)]
struct Artifact {
    archive_download_url: String,
}

#[derive(Deserialize)]
struct ArtifactList {
    artifacts: Vec<Artifact>,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRunAction {
    Requested,
    Completed,
    InProgress,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRunConclusion {
    Success,
    Failure,
    Neutral,
    Cancelled,
    TimedOut,
    ActionRequired,
    Stale,
    Skipped,
}

#[derive(Deserialize)]
pub struct WorkflowRun {
    pub head_branch: String,
    pub head_sha: String,
    pub status: WorkflowRunAction,
    pub conclusion: Option<WorkflowRunConclusion>,
    pub artifacts_url: String,
    pub path: PathBuf,
}

#[derive(Deserialize)]
pub struct WorkflowRunEvent {
    pub action: WorkflowRunAction,
    pub workflow_run: WorkflowRun,
}

pub struct Signature {
    pub digest: Vec<u8>,
}

static GITHUB_SHA256_HEADER_NAME: HeaderName = HeaderName::from_static("x-hub-signature-256");

impl Header for Signature {
    fn name() -> &'static HeaderName {
        &GITHUB_SHA256_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, HeaderError>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let header_value = values.next().ok_or_else(headers::Error::invalid);
        let hex_string = match header_value {
            Ok(value) => match value.to_str() {
                Ok(s) => s,
                Err(_) => return Err(HeaderError::invalid()),
            },
            Err(_) => return Err(HeaderError::invalid()),
        };

        match hex::decode(&hex_string[7..]) {
            Ok(digest) => Ok(Signature { digest }),
            Err(_) => Err(HeaderError::invalid()),
        }
    }

    fn encode<E: Extend<HeaderValue>>(&self, _values: &mut E) {
        unimplemented!()
    }
}

pub async fn download_and_extract_github_artifact(
    azure: &Azure,
    artifact_list_url: &str,
    download_path: &str,
) -> color_eyre::Result<()> {
    let token = azure.get_secret("github-api-token").await?;

    let mut token = header::HeaderValue::from_str(&format!("Bearer {}", token))?;
    token.set_sensitive(true);
    let mut default_headers = header::HeaderMap::new();
    default_headers.insert(header::AUTHORIZATION, token);
    default_headers.insert(header::USER_AGENT, HeaderValue::from_str("NuecesHomies")?);

    let client = reqwest::Client::builder()
        .default_headers(default_headers)
        .build()?;

    let list_text = client.get(artifact_list_url).send().await?.text().await?;
    let list = serde_json::from_str::<ArtifactList>(&list_text)?;
    let &artifact = &list
        .artifacts
        .first()
        .ok_or_else(|| eyre!("No artifacts found"))?;

    let zip_file = client
        .get(&artifact.archive_download_url)
        .send()
        .await?
        .bytes()
        .await?;

    let mut file = NamedTempFile::new().wrap_err("failed to create temp download file")?;
    file.write_all(zip_file.as_ref())
        .wrap_err("failed to write zip file")?;
    unzip(file.path(), download_path.as_ref()).wrap_err("failed to unzip file")?;

    Ok(())
}
