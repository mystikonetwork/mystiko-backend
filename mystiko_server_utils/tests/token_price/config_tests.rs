use mystiko_server_utils::token_price::config::TokenPriceConfig;
use std::env;
use std::path::PathBuf;

#[tokio::test]
async fn test_read_config() {
    let cfg = TokenPriceConfig::new(true, None).unwrap();
    assert_eq!(cfg.price_cache_ttl, Some(72000));
    assert_eq!(cfg.token_price.get(&5990), Some(0.01).as_ref());

    let cfg = TokenPriceConfig::new(false, None).unwrap();
    assert_eq!(cfg.price_cache_ttl, Some(1800));

    let cfg = TokenPriceConfig::new(false, Some(PathBuf::from("tests/token_price/files/config"))).unwrap();
    assert_eq!(cfg.price_cache_ttl, Some(90000));

    env::set_var("MYSTIKO_TOKEN_PRICE.PRICE_CACHE_TTL", "800");
    let cfg = TokenPriceConfig::new(false, None).unwrap();
    assert_eq!(cfg.price_cache_ttl, Some(800));

    let cfg = TokenPriceConfig::new(false, Some(PathBuf::from("tests/token_price/files/config"))).unwrap();
    assert_eq!(cfg.price_cache_ttl, Some(800));
    let tokens = cfg.tokens();
    assert!(tokens.len() > 1);
}
