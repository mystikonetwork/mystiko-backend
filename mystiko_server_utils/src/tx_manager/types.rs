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
    fn tx_eip1559(&self) -> bool;
    async fn gas_price(&self, provider: &Provider<P>) -> TransactionMiddlewareResult<U256>;
    async fn estimate_gas(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<U256>;
    async fn send(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<TxHash>;
    async fn confirm(
        &self,
        tx_hash: &TxHash,
        provider: &Provider<P>,
    ) -> TransactionMiddlewareResult<TransactionReceipt>;
}

#[async_trait::async_trait]
impl<P> TransactionMiddleware<P> for Box<dyn TransactionMiddleware<P>>
where
    P: JsonRpcClient,
{
    fn tx_eip1559(&self) -> bool {
        self.as_ref().tx_eip1559()
    }

    async fn gas_price(&self, provider: &Provider<P>) -> TransactionMiddlewareResult<U256> {
        self.as_ref().gas_price(provider).await
    }

    async fn estimate_gas(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<U256> {
        self.as_ref().estimate_gas(data, provider).await
    }

    async fn send(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<TxHash> {
        self.as_ref().send(data, provider).await
    }

    async fn confirm(
        &self,
        tx_hash: &TxHash,
        provider: &Provider<P>,
    ) -> TransactionMiddlewareResult<TransactionReceipt> {
        self.as_ref().confirm(tx_hash, provider).await
    }
}
