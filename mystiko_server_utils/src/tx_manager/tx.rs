use crate::tx_manager::config::{TxManagerChainConfig, TxManagerConfig};
use crate::tx_manager::error::{Result, TransactionMiddlewareError};
use ethers_core::types::transaction::eip1559::Eip1559TransactionRequest;
use ethers_core::types::transaction::eip2718::TypedTransaction;
use ethers_core::types::transaction::request::TransactionRequest;
use ethers_core::types::transaction::response::TransactionReceipt;
use ethers_core::types::{BlockNumber, TxHash, U256, U64};
use log::{info, warn};
// use ethers_middleware::gas_escalator::GasEscalatorMiddleware;
// use ethers_middleware::gas_escalator::{Frequency, GeometricGasPrice};
use crate::tx_manager::gas::eip1559_default_estimator;
use crate::tx_manager::types::{TransactionData, TransactionMiddleware, TransactionMiddlewareResult};
use ethers_middleware::gas_oracle::{GasOracle, ProviderOracle};
use ethers_middleware::{NonceManagerMiddleware, SignerMiddleware};
use ethers_providers::{JsonRpcClient, Middleware, Provider};
use ethers_signers::{LocalWallet, Signer};
use std::cmp::{max, min};
use std::marker::PhantomData;
use std::ops::{Div, Mul};
use std::time::Duration;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, TypedBuilder)]
pub struct TxManagerBuilder {
    chain_id: u64,
    config: TxManagerConfig,
    wallet: LocalWallet,
}

#[derive(Debug)]
pub struct TxManager<P> {
    chain_id: u64,
    tx_eip1559: bool,
    config: TxManagerChainConfig,
    wallet: LocalWallet,
    _marker: PhantomData<P>,
}

impl TxManagerBuilder {
    pub async fn build<P: JsonRpcClient>(
        &self,
        tx_eip1559: Option<bool>,
        provider: &Provider<P>,
    ) -> TransactionMiddlewareResult<TxManager<P>> {
        let chain_config: TxManagerChainConfig = self.config.chain_config(&self.chain_id)?;
        let tx_eip1559 = match tx_eip1559 {
            Some(eip1559) => eip1559,
            None => provider.estimate_eip1559_fees(None).await.is_ok(),
        };
        Ok(TxManager {
            chain_id: self.chain_id,
            config: chain_config.clone(),
            wallet: self.wallet.clone(),
            tx_eip1559,
            _marker: Default::default(),
        })
    }
}

#[async_trait::async_trait]
impl<P> TransactionMiddleware<P> for TxManager<P>
where
    P: JsonRpcClient + Send + Sync,
{
    fn tx_eip1559(&self) -> bool {
        self.tx_eip1559
    }

    async fn gas_price(&self, provider: &Provider<P>) -> TransactionMiddlewareResult<U256> {
        if self.tx_eip1559 {
            let (max_fee_per_gas, priority_fee) = self.gas_price_1559_tx(provider).await?;
            Ok(max_fee_per_gas + priority_fee)
        } else {
            self.gas_price_legacy_tx(provider).await
        }
    }

    async fn estimate_gas(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<U256> {
        let typed_tx = match self.tx_eip1559 {
            true => {
                let priority_fee = self.config.min_priority_fee_per_gas.unwrap_or_else(|| 10000000_u64);
                let tx = self.build_1559_tx(data, &priority_fee.into(), provider).await?;
                TypedTransaction::Eip1559(tx)
            }
            false => {
                let tx = self.build_legacy_tx(data, provider).await?;
                TypedTransaction::Legacy(tx)
            }
        };

        let signer = SignerMiddleware::new(provider, self.wallet.clone());
        let gas = signer
            .estimate_gas(&typed_tx, None)
            .await
            .map_err(|why| TransactionMiddlewareError::EstimateGasError(why.to_string()))?;
        if gas.is_zero() {
            return Err(TransactionMiddlewareError::EstimateGasError(
                "estimate gas is zero".to_string(),
            ));
        }
        Ok(gas)
    }

    async fn send(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<TxHash> {
        info!(
            "send tx to {:?} with gas {:?} and gas_price {:?}",
            data.to, data.gas, data.max_price
        );
        self.send_tx(data, provider).await
    }

    async fn confirm(
        &self,
        tx_hash: &TxHash,
        provider: &Provider<P>,
    ) -> TransactionMiddlewareResult<TransactionReceipt> {
        let max_count = self.config.get_max_confirm_count(self.chain_id);
        info!("confirm tx {:?} with max wait count {:?}", tx_hash, max_count);
        self.confirm_tx(tx_hash, max_count, provider).await
    }
}

impl<P> TxManager<P>
where
    P: JsonRpcClient + Send + Sync,
{
    async fn send_tx(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<TxHash> {
        let gas_limit = data.gas * (100 + self.config.gas_limit_reserve_percentage) / 100;
        if self.tx_eip1559 {
            let (max_fee_per_gas, priority_fee) = self.gas_price_1559_tx(provider).await?;
            if max_fee_per_gas + priority_fee > data.max_price {
                return Err(TransactionMiddlewareError::GasPriceError(format!(
                    "gas price too high provider max_fee_per_gas: {:?} priority_fee: {:?} data max_price: {:?}",
                    max_fee_per_gas, priority_fee, data.max_price
                )));
            }
            let mut tx_request = self.build_1559_tx(data, &priority_fee, provider).await?;
            tx_request.gas = Some(gas_limit);
            self.send_1559_tx(tx_request, provider).await
        } else {
            let mut tx_request = self.build_legacy_tx(data, provider).await?;
            tx_request.gas = Some(gas_limit);

            if self.config.lower_gas_price_mod {
                let result = self
                    .try_send_lower_gas_price_tx(data, tx_request.clone(), provider)
                    .await;
                match result {
                    Ok(tx_hash) => return Ok(tx_hash),
                    Err(e) => warn!("try send lower gas price tx failed: {:?}", e),
                }
            }

            self.send_legacy_tx(tx_request, provider).await
        }
    }

    async fn try_send_lower_gas_price_tx(
        &self,
        data: &TransactionData,
        tx_request: TransactionRequest,
        provider: &Provider<P>,
    ) -> TransactionMiddlewareResult<TxHash> {
        let mut tx = tx_request;
        let percentage = self.config.get_lower_gas_price_percentage(self.chain_id);
        let gas_price = data.max_price.mul(percentage).div(100);
        info!(
            "try send lower gas price transaction with gas price: {:?}, nonce: {:?}",
            gas_price, tx.nonce
        );
        tx.gas_price = Some(gas_price);
        let tx_hash = self.send_legacy_tx(tx, provider).await?;
        let max_count = self.config.get_lower_gas_price_confirm_count(self.chain_id);
        info!(
            "try confirm lower gas price tx {:?} with max wait count {:?}",
            tx_hash, max_count
        );
        let _ = self.confirm_tx(&tx_hash, max_count, provider).await?;
        Ok(tx_hash)
    }

    async fn confirm_tx(
        &self,
        tx_hash: &TxHash,
        max_count: u32,
        provider: &Provider<P>,
    ) -> TransactionMiddlewareResult<TransactionReceipt> {
        for _ in 0..max_count {
            tokio::time::sleep(Duration::from_secs(self.config.confirm_interval_secs)).await;
            let tx_first = provider.get_transaction(*tx_hash).await;

            // try again for some provider error of lose transaction for a while
            let tx = match tx_first {
                Ok(Some(t)) => t,
                Ok(None) => {
                    warn!("Transaction not found, retrying...");
                    tokio::time::sleep(Duration::from_secs(self.config.confirm_interval_secs)).await;
                    provider
                        .get_transaction(*tx_hash)
                        .await
                        .map_err(|why| TransactionMiddlewareError::ConfirmTxError(why.to_string()))?
                        .ok_or_else(|| TransactionMiddlewareError::TxDroppedError(tx_hash.to_string()))?
                }
                Err(why) => {
                    warn!("get transaction meet error: {:?}, retrying...", why.to_string());
                    tokio::time::sleep(Duration::from_secs(self.config.confirm_interval_secs)).await;
                    provider
                        .get_transaction(*tx_hash)
                        .await
                        .map_err(|why| TransactionMiddlewareError::ConfirmTxError(why.to_string()))?
                        .ok_or_else(|| TransactionMiddlewareError::TxDroppedError(tx_hash.to_string()))?
                }
            };

            if let Some(tx_block_number) = tx.block_number {
                let current_block_number = match provider.get_block_number().await {
                    Ok(block_number) => block_number,
                    Err(e) => {
                        warn!("Error fetching current block number: {:?}, retrying...", e.to_string());
                        continue;
                    }
                };

                if current_block_number < tx_block_number.saturating_add(self.config.confirm_blocks.into()) {
                    info!("Waiting for transaction to be confirmed...");
                    continue;
                }
            } else {
                continue;
            }

            let receipt = match provider.get_transaction_receipt(*tx_hash).await {
                Ok(receipt) => receipt,
                Err(e) => {
                    warn!("Error fetching transaction receipt: {:?}, retrying...", e.to_string());
                    continue;
                }
            };

            if let Some(receipt) = receipt {
                if receipt.status != Some(U64::from(1)) {
                    return Err(TransactionMiddlewareError::ConfirmTxError(format!(
                        "failed: {:?}",
                        receipt
                    )));
                }
                return Ok(receipt);
            }
        }

        Err(TransactionMiddlewareError::ConfirmTxError(format!(
            "reach max confirm count: {:?}",
            max_count
        )))
    }

    async fn gas_price_1559_tx(&self, provider: &Provider<P>) -> Result<(U256, U256)> {
        let (max_fee_per_gas, mut priority_fee) = provider
            .estimate_eip1559_fees(Some(eip1559_default_estimator))
            .await
            .map_err(|e| TransactionMiddlewareError::GasPriceError(e.to_string()))?;
        priority_fee = self
            .config
            .min_priority_fee_per_gas
            .map_or_else(|| priority_fee, |cfg_min| max(cfg_min.into(), priority_fee));
        priority_fee = self
            .config
            .max_priority_fee_per_gas
            .map_or_else(|| priority_fee, |cfg_max| min(cfg_max.into(), priority_fee));
        Ok((max_fee_per_gas, priority_fee))
    }

    async fn gas_price_legacy_tx(&self, provider: &Provider<P>) -> Result<U256> {
        let gas_oracle = ProviderOracle::new(provider);
        let mut gas_price = gas_oracle
            .fetch()
            .await
            .map_err(|e| TransactionMiddlewareError::GasPriceError(e.to_string()))?;
        gas_price = self
            .config
            .min_gas_price
            .map_or_else(|| gas_price, |cfg_min| max(cfg_min.into(), gas_price));
        Ok(gas_price)
    }

    async fn build_legacy_tx(&self, data: &TransactionData, provider: &Provider<P>) -> Result<TransactionRequest> {
        let cur_nonce = self.get_current_nonce(provider).await?;

        Ok(TransactionRequest::new()
            .chain_id(self.chain_id)
            .to(ethers_core::types::NameOrAddress::Address(data.to))
            .value(data.value)
            .data(data.data.to_vec())
            .nonce(cur_nonce)
            .gas_price(data.max_price))
    }

    async fn send_legacy_tx(&self, tx_request: TransactionRequest, provider: &Provider<P>) -> Result<TxHash> {
        let signer = SignerMiddleware::new(provider, self.wallet.clone());
        // todo support gas escalator
        // let geometric_escalator = GeometricGasPrice::new(
        //     // self.config.gas_price_coefficient,
        //     // self.config.gas_price_every_secs,
        //     5.0,
        //     10u64,
        //     Some(self.choose_max_gas_price()),
        // );
        //
        // let gas_escalator = GasEscalatorMiddleware::new(
        //     signer,
        //     geometric_escalator,
        //     // Frequency::Duration(self.config.bump_gas_price_ms),
        //     Frequency::Duration(300),
        // );
        info!("send legacy tx with gas price {:?}", tx_request.gas_price);
        let pending_tx = signer
            .send_transaction(tx_request, None)
            .await
            .map_err(|why| TransactionMiddlewareError::SendTxError(why.to_string()))?;

        Ok(pending_tx.tx_hash())
    }

    async fn build_1559_tx(
        &self,
        data: &TransactionData,
        priority_fee: &U256,
        provider: &Provider<P>,
    ) -> Result<Eip1559TransactionRequest> {
        let cur_nonce = self.get_current_nonce(provider).await?;

        // todo set priority_fee_per_gas from provider
        Ok(Eip1559TransactionRequest::new()
            .chain_id(self.chain_id)
            .to(ethers_core::types::NameOrAddress::Address(data.to))
            .value(data.value)
            .data(data.data.to_vec())
            .nonce(cur_nonce)
            .max_fee_per_gas(data.max_price)
            .max_priority_fee_per_gas(*priority_fee))
    }

    async fn send_1559_tx(&self, tx_request: Eip1559TransactionRequest, provider: &Provider<P>) -> Result<TxHash> {
        let signer = SignerMiddleware::new(provider, self.wallet.clone());
        info!(
            "send 1559 tx with max_fee_per_gas {:?} and max_priority_fee_per_gas {:?} ",
            tx_request.max_fee_per_gas, tx_request.max_priority_fee_per_gas
        );
        let pending_tx = signer
            .send_transaction(tx_request, None)
            .await
            .map_err(|why| TransactionMiddlewareError::SendTxError(why.to_string()))?;
        Ok(pending_tx.tx_hash())
    }

    async fn get_current_nonce(&self, provider: &Provider<P>) -> Result<U256> {
        let nonce_manager = NonceManagerMiddleware::new(provider, self.wallet.address());
        nonce_manager
            .get_transaction_count(self.wallet.address(), Some(BlockNumber::Pending.into()))
            .await
            .map_err(|why| TransactionMiddlewareError::NonceError(why.to_string()))
    }
}
