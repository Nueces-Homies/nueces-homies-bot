use reqwest::header::InvalidHeaderValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum TMDBClientError {
    #[error("API token has illegal characters")]
    #[serde(skip)]
    ApiTokenError(#[from] InvalidHeaderValue),

    #[error("error making request")]
    #[serde(skip)]
    InternalClientError(#[from] reqwest::Error),

    #[error("json error")]
    #[serde(skip)]
    ClientDeserializationError(#[from] serde_json::Error),
}
