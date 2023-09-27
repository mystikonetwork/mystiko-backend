use crate::tx_manager::config::{TxManagerChainConfig, TxManagerConfig};
use crate::tx_manager::error::{Result, TransactionMiddlewareError};
use ethers_core::types::transaction::eip1559::Eip1559TransactionRequest;
use ethers_core::types::transaction::eip2718::TypedTransaction;
use ethers_core::types::transaction::request::TransactionRequest;
use ethers_core::types::transaction::response::TransactionReceipt;
use ethers_core::types::{BlockNumber, TxHash, U256, U64};
// use ethers_middleware::gas_escalator::GasEscalatorMiddleware;
// use ethers_middleware::gas_escalator::{Frequency, GeometricGasPrice};
use crate::tx_manager::types::{TransactionData, TransactionMiddleware, TransactionMiddlewareResult};
use ethers_middleware::gas_oracle::{GasOracle, ProviderOracle};
use ethers_middleware::{NonceManagerMiddleware, SignerMiddleware};
use ethers_providers::{JsonRpcClient, Middleware, Provider};
use ethers_signers::{LocalWallet, Signer};
use std::marker::PhantomData;
use std::time::Duration;
use tracing::info;
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
    config: TxManagerChainConfig,
    wallet: LocalWallet,
    support_1559: bool,
    _marker: PhantomData<P>,
}

impl TxManagerBuilder {
    pub async fn build<P: JsonRpcClient>(&self, provider: &Provider<P>) -> TransactionMiddlewareResult<TxManager<P>> {
        let chain_config = self.config.chain_config(&self.chain_id)?;
        let support_1559 =
            !chain_config.force_gas_price && ProviderOracle::new(provider).estimate_eip1559_fees().await.is_ok();
        Ok(TxManager {
            chain_id: self.chain_id,
            config: chain_config.clone(),
            wallet: self.wallet.clone(),
            support_1559,
            _marker: Default::default(),
        })
    }
}

#[async_trait::async_trait]
impl<P> TransactionMiddleware<P> for TxManager<P>
where
    P: JsonRpcClient + Send + Sync,
{
    fn support_1559(&self) -> bool {
        self.support_1559
    }

    async fn gas_price(&self, provider: &Provider<P>) -> TransactionMiddlewareResult<U256> {
        if self.support_1559 {
            let (max_fee_per_gas, priority_fee) = self.gas_price_1559_tx(provider).await?;
            Ok(max_fee_per_gas + priority_fee)
        } else {
            self.gas_price_legacy_tx(provider).await
        }
    }

    async fn estimate_gas(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<U256> {
        let typed_tx = match self.support_1559 {
            true => {
                let priority_fee = self.config.min_priority_fee_per_gas;
                let tx = self.build_1559_tx(data, &priority_fee.into(), provider).await?;
                TypedTransaction::try_from(tx).expect("Failed to convert Eip1559TransactionRequest to TypedTransaction")
            }
            false => {
                let tx = self.build_legacy_tx(data, provider).await?;
                TypedTransaction::try_from(tx).expect("Failed to convert TransactionRequest to TypedTransaction")
            }
        };

        let signer = SignerMiddleware::new(provider, self.wallet.clone());
        signer
            .estimate_gas(&typed_tx, None)
            .await
            .map_err(|why| TransactionMiddlewareError::EstimateGasError(why.to_string()))
    }

    async fn send(&self, data: &TransactionData, provider: &Provider<P>) -> TransactionMiddlewareResult<TxHash> {
        info!(
            "send tx to {:?} with gas {:?} and gas_price {:?}",
            data.to, data.gas, data.max_price
        );

        let gas_limit = data.gas * (100 + self.config.gas_limit_reserve_percentage) / 100;
        if self.support_1559 {
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
            // todo change gas
            let gas_price = self.gas_price_legacy_tx(provider).await?;
            if gas_price > data.max_price {
                return Err(TransactionMiddlewareError::GasPriceError(format!(
                    "gas price too high provider gas_price: {:?} data max_price: {:?}",
                    gas_price, data.max_price
                )));
            }
            let mut tx_request = self.build_legacy_tx(data, provider).await?;
            tx_request.gas = Some(gas_limit);
            self.send_legacy_tx(tx_request, provider).await
        }
    }

    async fn confirm(
        &self,
        tx_hash: &TxHash,
        provider: &Provider<P>,
    ) -> TransactionMiddlewareResult<TransactionReceipt> {
        info!("confirm tx {:?}", tx_hash);

        for _ in 0..self.config.max_confirm_count {
            tokio::time::sleep(Duration::from_secs(self.config.confirm_interval_secs)).await;
            let tx_first = provider
                .get_transaction(*tx_hash)
                .await
                .map_err(|why| TransactionMiddlewareError::ConfirmTxError(why.to_string()))?;

            // try again for some provider error of lose transaction for a while
            // todo polygon provider bug, more wait time
            let tx = match tx_first {
                Some(t) => t,
                None => {
                    tokio::time::sleep(Duration::from_secs(self.config.confirm_interval_secs)).await;
                    provider
                        .get_transaction(*tx_hash)
                        .await
                        .map_err(|why| TransactionMiddlewareError::ConfirmTxError(why.to_string()))?
                        .ok_or_else(|| TransactionMiddlewareError::TxDroppedError(tx_hash.to_string()))?
                }
            };

            if let Some(block_number) = tx.block_number {
                let current_block_number = provider
                    .get_block_number()
                    .await
                    .map_err(|why| TransactionMiddlewareError::ConfirmTxError(why.to_string()))?;
                if current_block_number < block_number.saturating_add(self.config.confirm_blocks.into()) {
                    info!("waiting for tx to be confirmed");
                    continue;
                }
            } else {
                continue;
            }

            let receipt = provider
                .get_transaction_receipt(*tx_hash)
                .await
                .map_err(|why| TransactionMiddlewareError::ConfirmTxError(why.to_string()))?;

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
            self.config.max_confirm_count
        )))
    }
}

impl<P> TxManager<P>
where
    P: JsonRpcClient + Send + Sync,
{
    async fn gas_price_1559_tx(&self, provider: &Provider<P>) -> Result<(U256, U256)> {
        let gas_oracle = ProviderOracle::new(provider);

        let (max_fee_per_gas, mut priority_fee) = gas_oracle
            .estimate_eip1559_fees()
            .await
            .map_err(|e| TransactionMiddlewareError::GasPriceError(e.to_string()))?;
        let cfg_min_priority_fee: U256 = self.config.min_priority_fee_per_gas.into();
        let cfg_max_priority_fee: U256 = self.config.max_priority_fee_per_gas.into();
        if priority_fee < cfg_min_priority_fee {
            priority_fee = cfg_min_priority_fee;
        } else if priority_fee > cfg_max_priority_fee {
            priority_fee = cfg_max_priority_fee;
        }

        Ok((max_fee_per_gas, priority_fee))
    }

    async fn gas_price_legacy_tx(&self, provider: &Provider<P>) -> Result<U256> {
        let gas_oracle = ProviderOracle::new(provider);

        gas_oracle
            .fetch()
            .await
            .map_err(|e| TransactionMiddlewareError::GasPriceError(e.to_string()))
    }

    async fn build_legacy_tx(&self, data: &TransactionData, provider: &Provider<P>) -> Result<TransactionRequest> {
        let curr_nonce = self.get_current_nonce(provider).await?;

        Ok(TransactionRequest::new()
            .chain_id(self.chain_id)
            .to(ethers_core::types::NameOrAddress::Address(data.to))
            .value(data.value)
            .data(data.data.to_vec())
            .nonce(curr_nonce)
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
        let curr_nonce = self.get_current_nonce(provider).await?;

        // todo set priority_fee_per_gas from provider
        Ok(Eip1559TransactionRequest::new()
            .chain_id(self.chain_id)
            .to(ethers_core::types::NameOrAddress::Address(data.to))
            .value(data.value)
            .data(data.data.to_vec())
            .nonce(curr_nonce)
            .max_fee_per_gas(data.max_price - priority_fee)
            .max_priority_fee_per_gas(*priority_fee))
    }

    async fn send_1559_tx(&self, tx_request: Eip1559TransactionRequest, provider: &Provider<P>) -> Result<TxHash> {
        let signer = SignerMiddleware::new(provider, self.wallet.clone());
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
