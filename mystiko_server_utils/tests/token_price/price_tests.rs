extern crate mystiko_fs;

use ethers_core::types::U256;
use mystiko_server_utils::token_price::config::TokenPriceConfig;
use mystiko_server_utils::token_price::{PriceMiddleware, TokenPrice};

const COIN_MARKET_API_KEY: &str = "";

#[tokio::test]
async fn test_get_token_id() {
    let default_cfg = TokenPriceConfig::new(false, None).unwrap();
    let tp = TokenPrice::new(&default_cfg, COIN_MARKET_API_KEY).unwrap();
    let id = tp.get_token_id("XZK").await.unwrap();
    assert_eq!(id, [30608]);
}

#[tokio::test]
async fn test_price() {
    let default_cfg = TokenPriceConfig::new(false, None).unwrap();
    let mut tp = TokenPrice::new(&default_cfg, COIN_MARKET_API_KEY).unwrap();
    let price = tp.price("ETH").await.unwrap();
    assert!(price > 100.0);
}

#[tokio::test]
async fn test_swap() {
    let default_cfg = TokenPriceConfig::new(false, None).unwrap();
    let mut tp = TokenPrice::new(&default_cfg, COIN_MARKET_API_KEY).unwrap();

    let amount = tp
        .swap("ETH", 18, U256::from("1000000000000000000"), "USDT", 6)
        .await
        .unwrap();
    assert!(amount > U256::from("100000000"));
}

#[tokio::test]
async fn test_price_by_times() {
    let default_cfg = TokenPriceConfig::new(false, None).unwrap();
    let tp = TokenPrice::new(&default_cfg, COIN_MARKET_API_KEY).unwrap();
    let price = tp.price_by_times("POL", 1733890861).await.unwrap();
    println!("price: {}", price);
    let price = tp.price_by_times("USDC", 1733890861).await.unwrap();
    println!("price: {}", price);
    let price = tp.price_by_times("ETH", 1733890861).await.unwrap();
    println!("price: {}", price);
    let price = tp.price_by_times("AAVE", 1733890861).await.unwrap();
    println!("price: {}", price);
    let price = tp.price_by_times("BUSD", 1733890861).await;
    assert!(price.is_err());
    let price = tp.price_by_times("USDT", 1733890861).await;
    assert!(price.is_err());
}
