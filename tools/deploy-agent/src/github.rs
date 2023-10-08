use std::io::Write;

use axum::headers;
use axum::headers::{Error, Header, HeaderName, HeaderValue};
use reqwest::header;
use serde::Deserialize;
use tempfile::NamedTempFile;

use crate::azure::Azure;
use crate::unzip::unzip;

#[derive(Debug)]
pub enum EventType {
    Ping,
    WorkflowRun,
}

#[derive(Debug)]
pub struct GitHubEvent(EventType);

static GITHUB_EVENT_HEADER_NAME: HeaderName = HeaderName::from_static("x-github-event");

impl Header for GitHubEvent {
    fn name() -> &'static HeaderName {
        &GITHUB_EVENT_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        if value == "ping" {
            Ok(GitHubEvent(EventType::Ping))
        } else if value == "workflow_run" {
            Ok(GitHubEvent(EventType::WorkflowRun))
        } else {
            Err(headers::Error::invalid())
        }
    }

    fn encode<E: Extend<HeaderValue>>(&self, _values: &mut E) {
        todo!()
    }
}

pub struct Signature {
    pub digest: Vec<u8>,
}

static GITHUB_SHA256_HEADER_NAME: HeaderName = HeaderName::from_static("x-hub-signature-256");

impl Header for Signature {
    fn name() -> &'static HeaderName {
        &GITHUB_SHA256_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        let stringy = value.to_str().unwrap().to_owned()[7..].to_owned();
        let digest = hex::decode(stringy).unwrap();
        Ok(Signature { digest })
    }

    fn encode<E: Extend<HeaderValue>>(&self, _values: &mut E) {
        todo!()
    }
}

#[derive(Deserialize)]
struct Artifact {
    archive_download_url: String,
}

#[derive(Deserialize)]
struct ArtifactList {
    artifacts: Vec<Artifact>,
}

pub async fn download_and_extract_github_artifact(
    azure: &Azure,
    artifact_list_url: &str,
    download_path: &str,
) -> anyhow::Result<()> {
    let token = azure.get_secret("github-api-token").await?;

    let mut token = header::HeaderValue::from_str(&format!("Bearer {}", token))?;
    token.set_sensitive(true);
    let mut default_headers = header::HeaderMap::new();
    default_headers.insert(header::AUTHORIZATION, token);
    default_headers.insert(header::USER_AGENT, HeaderValue::from_str("NuecesHomies")?);

    let client = reqwest::Client::builder()
        .default_headers(default_headers)
        .build()?;

    let url = &artifact_list_url[1..artifact_list_url.len() - 1];
    let list_text = client.get(url).send().await?.text().await?;
    let list = serde_json::from_str::<ArtifactList>(&list_text)?;
    let &artifact = &list.artifacts.first().unwrap();

    let zip_file = client
        .get(&artifact.archive_download_url)
        .send()
        .await?
        .bytes()
        .await?;

    let mut file = NamedTempFile::new()?;
    file.write_all(zip_file.as_ref())?;
    unzip(file.path(), download_path.as_ref())?;

    Ok(())
}
