use thiserror::Error;

#[derive(Error, Debug)]
pub enum IGDBClientError {
    #[error("bad query submitted")]
    BadQueryError { query: String, endpoint: String },

    #[error("bad request")]
    BadRequestError,

    #[error("error making request")]
    InternalClientError(#[from] reqwest::Error),

    #[error("Unable to get token")]
    TokenError,

    #[error("Unable to decode response")]
    ResponseDecodeError(#[from] prost::DecodeError),
}
