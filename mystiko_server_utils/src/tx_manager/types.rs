use crate::tx_manager::TransactionMiddlewareError;
use ethers_core::types::{Address, Bytes, TransactionReceipt, TxHash, U256};
use ethers_providers::{JsonRpcClient, Provider};
use std::fmt::Debug;
use typed_builder::TypedBuilder;

pub type TransactionMiddlewareResult<T> = anyhow::Result<T, TransactionMiddlewareError>;

#[derive(Debug, Clone, TypedBuilder)]
pub struct TransactionData {
    pub to: Address,
    pub data: Bytes,
    pub value: U256,
    pub gas: U256,
    pub max_price: U256,
}

#[async_trait::async_trait]
pub trait TransactionMiddleware<P: JsonRpcClient>: Debug + Send + Sync {
    fn support_1559(&self) -> bool;
    async fn gas_price(&self, provider: &Provider<P>) -> TransactionMiddlewareResult<U256>;
    async fn estimate_gas(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<U256>;
    async fn send(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<TxHash>;
    async fn confirm(
        &self,
        tx_hash: &TxHash,
        provider: &Provider<P>,
    ) -> TransactionMiddlewareResult<TransactionReceipt>;
}
