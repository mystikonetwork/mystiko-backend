use crate::token_price::config::TokenPriceConfig;
use crate::token_price::error::PriceMiddlewareError;
use crate::token_price::query::QueryApiInstance;
use crate::token_price::utils::{calc_token_precision, f64_to_u256, u256_to_f64};
use crate::token_price::{PriceMiddleware, PriceMiddlewareResult};
use ethers_core::types::U256;
use log::warn;
use std::collections::HashMap;
use std::ops::{Div, Mul};
use std::time::SystemTime;
use tokio::sync::RwLock;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct TokenPriceData {
    record_time: SystemTime,
    // prices hash map key token id
    prices: HashMap<u32, f64>,
}

#[derive(Debug)]
pub struct TokenPrice {
    config: TokenPriceConfig,
    instance: QueryApiInstance,
    ids: Vec<u32>,
    data: RwLock<TokenPriceData>,
}

#[async_trait::async_trait]
impl PriceMiddleware for TokenPrice {
    async fn price(&self, symbol: &str) -> PriceMiddlewareResult<f64> {
        self.try_update_token_prices().await?;
        self.get_token_price(symbol).await
    }

    async fn price_by_times(&self, symbol: &str, timestamp_second: u64) -> PriceMiddlewareResult<f64> {
        let token_symbol = format!("{}USDT", symbol.to_string().to_uppercase());
        let response = reqwest::get(format!(
            "https://api.binance.com/api/v3/klines?symbol={}&interval=1s&limit=1&startTime={}&endTime={}",
            token_symbol,
            timestamp_second * 1000,
            timestamp_second * 1000
        ))
        .await?;
        let price = response.json::<Vec<Vec<serde_json::Value>>>().await?;
        let price_str = price[0][4]
            .as_str()
            .ok_or_else(|| PriceMiddlewareError::ParsePriceError(format!("Invalid price format: {:?}", price[0][4])))?;
        Ok(price_str
            .parse::<f64>()
            .map_err(|_| PriceMiddlewareError::ParsePriceError(price_str.to_string()))?)
    }

    async fn swap(
        &self,
        asset_a: &str,
        decimal_a: u32,
        amount_a: U256,
        asset_b: &str,
        decimal_b: u32,
    ) -> PriceMiddlewareResult<U256> {
        self.try_update_token_prices().await?;

        let price_a = self.get_token_price(asset_a).await?;
        let price_b = self.get_token_price(asset_b).await?;
        let mut amount = u256_to_f64(amount_a, decimal_a);
        amount = amount.mul(price_a).div(price_b);
        let mut amount = f64_to_u256(amount, decimal_b);
        let token_precision = calc_token_precision(price_b, decimal_b, self.config.swap_precision);
        amount /= token_precision;
        amount *= token_precision;

        Ok(amount)
    }
}

impl TokenPrice {
    pub fn new(cfg: &TokenPriceConfig, api_key: &str) -> PriceMiddlewareResult<Self> {
        let instance = QueryApiInstance::new(api_key, cfg.base_url.clone(), cfg.query_timeout_secs)?;
        let data = TokenPriceData::builder()
            .record_time(SystemTime::UNIX_EPOCH)
            .prices(cfg.token_price.clone())
            .build();
        Ok(TokenPrice {
            ids: cfg.ids(),
            config: cfg.clone(),
            instance,
            data: RwLock::new(data),
        })
    }

    pub async fn get_token_id(&self, symbol: &str) -> PriceMiddlewareResult<Vec<u32>> {
        self.instance.get_token_id(symbol).await
    }

    async fn try_update_token_prices(&self) -> PriceMiddlewareResult<()> {
        if self.should_do_update().await? {
            if let Err(e) = self.update_token_prices().await {
                warn!("update token price failed: {:?}", e);
            }
        }
        Ok(())
    }

    async fn should_do_update(&self) -> PriceMiddlewareResult<bool> {
        let system_now = SystemTime::now();
        let data = self.data.read().await;
        let current = system_now
            .duration_since(data.record_time)
            .map_err(|_| PriceMiddlewareError::InternalError)?
            .as_secs();
        Ok(current >= self.config.price_cache_ttl())
    }

    async fn update_token_prices(&self) -> PriceMiddlewareResult<()> {
        let token_prices = self.instance.get_latest_price(&self.ids).await?;
        let mut data = self.data.write().await;
        for (key, price) in token_prices {
            data.prices.insert(key, price);
        }
        data.record_time = SystemTime::now();
        Ok(())
    }

    async fn get_token_price(&self, symbol: &str) -> PriceMiddlewareResult<f64> {
        let data = self.data.read().await;
        self.config
            .coin_market_cap_ids
            .get(symbol)
            .ok_or(PriceMiddlewareError::TokenNotSupportError(symbol.to_string()))
            .and_then(|id| {
                data.prices
                    .get(id)
                    .copied()
                    .ok_or(PriceMiddlewareError::TokenPriceNotInitError(symbol.to_string()))
            })
    }
}
