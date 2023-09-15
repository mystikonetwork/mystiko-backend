use crate::token_price::PriceMiddlewareError;
use ethers_core::types::U256;

pub type PriceMiddlewareResult<T> = anyhow::Result<T, PriceMiddlewareError>;

#[async_trait::async_trait]
pub trait PriceMiddleware {
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
