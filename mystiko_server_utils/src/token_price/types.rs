use crate::token_price::PriceMiddlewareError;
use ethers_core::types::U256;
use std::fmt::Debug;
use async_trait::async_trait;

pub type PriceMiddlewareResult<T> = anyhow::Result<T, PriceMiddlewareError>;

#[async_trait::async_trait]
pub trait PriceMiddleware: Debug + Send + Sync {
    async fn price(&self, symbol: &str) -> PriceMiddlewareResult<f64>;
    async fn swap(
        &self,
        asset_a: &str,
        decimal_a: u32,
        amount_a: U256,
        asset_b: &str,
        decimal_b: u32,
    ) -> PriceMiddlewareResult<U256>;
}

#[async_trait]
impl PriceMiddleware for Box<dyn PriceMiddleware> {
    async fn price(&self, symbol: &str) -> PriceMiddlewareResult<f64> {
        self.as_ref().price(symbol).await
    }

    async fn swap(
        &self,
        asset_a: &str,
        decimal_a: u32,
        amount_a: U256,
        asset_b: &str,
        decimal_b: u32,
    ) -> PriceMiddlewareResult<U256> {
        self.as_ref().swap(asset_a, decimal_a, amount_a, asset_b, decimal_b).await
    }
}
