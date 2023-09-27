use anyhow::Error as AnyhowError;
use reqwest::header::InvalidHeaderValue;
use serde_json::Error as SerdeJsonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PriceMiddlewareError {
    #[error("read file error {0}")]
    FileError(String),
    #[error("api key not configure error")]
    ApiKeyNotConfigureError,
    #[error("token: {0} not support error")]
    TokenNotSupportError(String),
    #[error("server response error code: {0}")]
    ResponseError(u64),
    #[error("internal error")]
    InternalError,
    #[error(transparent)]
    SerdeJsonError(#[from] SerdeJsonError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[error(transparent)]
    AnyhowError(#[from] AnyhowError),
}
