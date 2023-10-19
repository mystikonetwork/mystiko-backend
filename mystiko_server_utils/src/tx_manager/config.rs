use crate::tx_manager::{TransactionMiddlewareError, TransactionMiddlewareResult};
use mystiko_utils::config::{load_config, ConfigFile, ConfigLoadOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use typed_builder::TypedBuilder;

const TX_MANAGER_ENV_CONFIG_PREFIX: &str = "MYSTIKO_TX_MANAGER";

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct TxManagerChainConfig {
    #[serde(default = "default_gas_limit_reserve_percentage")]
    #[builder(default = default_gas_limit_reserve_percentage())]
    pub gas_limit_reserve_percentage: u32,

    #[serde(default)]
    #[builder(default)]
    pub min_priority_fee_per_gas: Option<u64>,

    #[serde(default)]
    #[builder(default)]
    pub max_priority_fee_per_gas: Option<u64>,

    #[serde(default)]
    #[builder(default)]
    pub force_gas_price: Option<bool>,

    #[serde(default = "default_confirm_interval_secs")]
    #[builder(default = default_confirm_interval_secs())]
    pub confirm_interval_secs: u64,

    #[serde(default = "default_confirm_blocks")]
    #[builder(default = default_confirm_blocks())]
    pub confirm_blocks: u32,

    #[serde(default = "default_max_confirm_count")]
    #[builder(default = default_max_confirm_count())]
    pub max_confirm_count: u32,
}

impl TxManagerChainConfig {
    pub fn validate(&self) -> TransactionMiddlewareResult<()> {
        if let (Some(max_fee), Some(min_fee)) = (self.max_priority_fee_per_gas, self.min_priority_fee_per_gas) {
            if max_fee < min_fee {
                return Err(TransactionMiddlewareError::ConfigError(
                    "max_priority_fee_per_gas must be greater than min_priority_fee_per_gas".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn get_force_gas_price(&self, chain_id: u64) -> bool {
        self.force_gas_price.unwrap_or(default_force_gas_price(chain_id))
    }
}

impl Default for TxManagerChainConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct TxManagerConfig {
    #[serde(default)]
    #[builder(default)]
    pub chains: HashMap<u64, TxManagerChainConfig>,
}

impl Default for TxManagerConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl TxManagerConfig {
    pub fn new(config_path: Option<PathBuf>) -> anyhow::Result<Self> {
        let config_file: Option<ConfigFile<PathBuf>> = config_path
            .map(|p| {
                if p.join("tx_manager.json").exists() {
                    Some(p.join("tx_manager").into())
                } else {
                    None
                }
            })
            .unwrap_or(None);
        let options = if let Some(file) = config_file {
            ConfigLoadOptions::<PathBuf>::builder()
                .paths(file)
                .env_prefix(TX_MANAGER_ENV_CONFIG_PREFIX.to_string())
                .build()
        } else {
            ConfigLoadOptions::<PathBuf>::builder()
                .env_prefix(TX_MANAGER_ENV_CONFIG_PREFIX.to_string())
                .build()
        };
        load_config::<PathBuf, Self>(&options)
    }

    pub fn chain_config(&self, chain_id: &u64) -> TransactionMiddlewareResult<TxManagerChainConfig> {
        let config = self
            .chains
            .get(chain_id)
            .unwrap_or(&TxManagerChainConfig::default())
            .clone();
        config.validate()?;
        Ok(config)
    }
}

fn default_force_gas_price(chain_id: u64) -> bool {
    match chain_id {
        56 | 97 | 250 | 4002 => true,
        _ => false,
    }
}

fn default_confirm_interval_secs() -> u64 {
    10
}

fn default_confirm_blocks() -> u32 {
    5
}

fn default_max_confirm_count() -> u32 {
    100
}

fn default_gas_limit_reserve_percentage() -> u32 {
    10
}
