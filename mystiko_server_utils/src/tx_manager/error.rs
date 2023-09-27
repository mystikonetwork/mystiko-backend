use ethers_providers::ProviderError;
use serde_json::Error as SerdeJsonError;
use thiserror::Error;

pub type Result<T> = anyhow::Result<T, TransactionMiddlewareError>;

#[derive(Error, Debug)]
pub enum TransactionMiddlewareError {
    #[error("config error {0}")]
    ConfigError(String),
    #[error(transparent)]
    SerdeJsonError(#[from] SerdeJsonError),
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    #[error("nonce manager error {0}")]
    NonceError(String),
    #[error("gas price error {0}")]
    GasPriceError(String),
    #[error("estimate gas error {0}")]
    EstimateGasError(String),
    #[error("send transaction error {0}")]
    SendTxError(String),
    #[error("transaction: {0} dropped error")]
    TxDroppedError(String),
    #[error("confirm transaction error {0}")]
    ConfirmTxError(String),
}
