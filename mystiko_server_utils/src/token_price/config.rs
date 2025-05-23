use crate::token_price::PriceMiddlewareResult;
use mystiko_utils::config::{load_config, ConfigFile, ConfigLoadOptions};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use typed_builder::TypedBuilder;

const TOKEN_PRICE_ENV_CONFIG_PREFIX: &str = "MYSTIKO_TOKEN_PRICE";

#[derive(Debug, Clone, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct TokenPriceConfig {
    #[serde(default = "default_url")]
    #[builder(default = default_url())]
    pub base_url: String,

    #[serde(default = "default_query_timeout_secs")]
    #[builder(default = default_query_timeout_secs())]
    pub query_timeout_secs: u32,

    #[serde(default)]
    #[builder(default)]
    pub price_cache_ttl: Option<u64>,

    #[serde(default = "default_swap_precision")]
    #[builder(default = default_swap_precision())]
    pub swap_precision: u32,

    #[serde(default = "default_coin_market_cap_ids")]
    #[builder(default = default_coin_market_cap_ids())]
    pub coin_market_cap_ids: HashMap<String, u32>,

    #[serde(default = "default_token_price")]
    #[builder(default = default_token_price())]
    pub token_price: HashMap<u32, f64>,
}

impl Default for TokenPriceConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl TokenPriceConfig {
    pub fn new(is_testnet: bool, config_path: Option<PathBuf>) -> PriceMiddlewareResult<Self> {
        let config_file: Option<ConfigFile<PathBuf>> = config_path
            .map(|p| {
                if p.join("token_price.json").exists() {
                    Some(p.join("token_price").into())
                } else {
                    None
                }
            })
            .unwrap_or(None);
        let options = if let Some(file) = config_file {
            ConfigLoadOptions::<PathBuf>::builder()
                .paths(file)
                .env_prefix(TOKEN_PRICE_ENV_CONFIG_PREFIX.to_string())
                .build()
        } else {
            ConfigLoadOptions::<PathBuf>::builder()
                .env_prefix(TOKEN_PRICE_ENV_CONFIG_PREFIX.to_string())
                .build()
        };

        let mut config = load_config::<PathBuf, Self>(&options)?;
        if config.price_cache_ttl.is_none() {
            config.price_cache_ttl = Some(default_price_cache_ttl(is_testnet));
        }

        if is_testnet {
            config.coin_market_cap_ids.insert("XZK".to_string(), 1839);
        }

        Ok(config)
    }

    pub fn price_cache_ttl(&self) -> u64 {
        self.price_cache_ttl.unwrap_or(default_price_cache_ttl(true))
    }

    pub fn tokens(&self) -> Vec<String> {
        Vec::from_iter(self.coin_market_cap_ids.keys().cloned())
    }

    pub fn ids(&self) -> Vec<u32> {
        let set: HashSet<u32> = self.coin_market_cap_ids.values().cloned().collect();
        set.into_iter().collect()
    }
}

fn default_url() -> String {
    "https://pro-api.coinmarketcap.com".to_string()
}

fn default_query_timeout_secs() -> u32 {
    5
}

fn default_price_cache_ttl(is_testnet: bool) -> u64 {
    if is_testnet {
        72000
    } else {
        1800
    }
}

fn default_swap_precision() -> u32 {
    3
}

fn default_coin_market_cap_ids() -> HashMap<String, u32> {
    let mut ids = HashMap::new();
    ids.insert("ETH".to_string(), 1027);
    ids.insert("mETH".to_string(), 1027);
    ids.insert("WETH".to_string(), 1027);
    ids.insert("WETHWormhole".to_string(), 1027);
    ids.insert("BNB".to_string(), 1839);
    ids.insert("mBNB".to_string(), 1839);
    ids.insert("FTM".to_string(), 3513);
    ids.insert("mFTM".to_string(), 3513);
    ids.insert("MATIC".to_string(), 3890);
    ids.insert("mMATIC".to_string(), 3890);
    ids.insert("POL".to_string(), 28321);
    ids.insert("DEV".to_string(), 5990);
    ids.insert("mDEV".to_string(), 5990);
    ids.insert("AVAX".to_string(), 5805);
    ids.insert("mAVAX".to_string(), 5805);
    ids.insert("USDT".to_string(), 825);
    ids.insert("USDC".to_string(), 3408);
    ids.insert("USDbC".to_string(), 3408);
    ids.insert("BUSD".to_string(), 4687);
    ids.insert("MTT".to_string(), 1839);
    ids.insert("mUSD".to_string(), 1839);
    ids.insert("XZK".to_string(), 30608);
    ids.insert("ZETA".to_string(), 21259);
    ids
}

fn default_token_price() -> HashMap<u32, f64> {
    let mut ids = HashMap::new();
    ids.insert(5990, 0.01);
    ids
}
