use ethers_core::types::{U256, U64};
use ethers_core::utils::Anvil;
use ethers_providers::{Http, Middleware, Provider};
use ethers_signers::{LocalWallet, Signer};
use mystiko_server_utils::tx_manager::config::TxManagerChainConfig;
use mystiko_server_utils::tx_manager::config::TxManagerConfig;
use mystiko_server_utils::tx_manager::{TransactionData, TransactionMiddleware, TxManagerBuilder};

#[tokio::test]
async fn test_send_1559_tx() {
    let anvil = Anvil::new().spawn();
    let endpoint = anvil.endpoint();

    let provider = Provider::<Http>::try_from(endpoint).unwrap();
    let chain_id = provider.get_chainid().await.unwrap();
    let wallet: LocalWallet = anvil.keys().first().unwrap().clone().into();
    let wallet = wallet.with_chain_id(chain_id.as_u64());
    let mut cfg = TxManagerConfig::new(None).unwrap();
    cfg.chains.insert(
        chain_id.as_u64(),
        TxManagerChainConfig::builder().confirm_blocks(0_u32).build(),
    );
    let to = anvil.addresses()[1];
    let value = ethers_core::utils::parse_ether("1").unwrap();

    let builder = TxManagerBuilder::builder()
        .config(cfg)
        .chain_id(chain_id.as_u64())
        .wallet(wallet)
        .build();
    let tx = builder.build(Some(true), &provider).await.unwrap();
    assert!(tx.tx_eip1559());

    let gas_price = tx.gas_price(&provider).await.unwrap();
    assert!(gas_price > U256::zero());

    let max_gas_price = U256::from(100_000_000_000u64);
    let mut tx_data = TransactionData::builder()
        .to(to)
        .data(vec![].into())
        .value(value)
        .gas(U256::zero())
        .max_price(max_gas_price)
        .build();
    let gas = tx.estimate_gas(&tx_data, &provider).await.unwrap();
    assert!(gas > U256::zero());

    let before = provider.get_balance(to, None).await.unwrap();
    tx_data.gas = gas;
    let tx_hash = tx.send(&tx_data, &provider).await.unwrap();
    let receipt = tx.confirm(&tx_hash, &provider).await.unwrap();
    assert_ne!(receipt.block_number.unwrap(), U64::from(0));
    assert_ne!(receipt.status.unwrap(), U64::from(0));
    assert_eq!(receipt.transaction_hash, tx_hash);
    let after = provider.get_balance(to, None).await.unwrap();
    assert_eq!(before + value, after);

    drop(anvil);
}

#[tokio::test]
async fn test_send_legacy_tx() {
    let anvil = Anvil::new().spawn();
    let endpoint = anvil.endpoint();

    let provider = Provider::<Http>::try_from(endpoint).unwrap();
    let chain_id = provider.get_chainid().await.unwrap().as_u64();
    let wallet: LocalWallet = anvil.keys().first().unwrap().clone().into();
    let wallet = wallet.with_chain_id(chain_id);

    let mut cfg = TxManagerConfig::new(None).unwrap();
    cfg.chains
        .insert(chain_id, TxManagerChainConfig::builder().confirm_blocks(0_u32).build());

    let to = anvil.addresses()[1];
    let value = ethers_core::utils::parse_ether("1").unwrap();

    let builder = TxManagerBuilder::builder()
        .config(cfg)
        .chain_id(chain_id)
        .wallet(wallet)
        .build();
    let tx = builder.build(Some(false), &provider).await.unwrap();
    assert!(!tx.tx_eip1559());

    let gas_price = tx.gas_price(&provider).await.unwrap();
    assert!(gas_price > U256::zero());

    let max_gas_price = U256::from(100_000_000_000u64);
    let mut tx_data = TransactionData::builder()
        .to(to)
        .data(vec![].into())
        .value(value)
        .gas(U256::zero())
        .max_price(max_gas_price)
        .build();
    let gas = tx.estimate_gas(&tx_data, &provider).await.unwrap();
    assert!(gas > U256::zero());

    let before = provider.get_balance(to, None).await.unwrap();
    tx_data.gas = gas;
    let tx_hash = tx.send(&tx_data, &provider).await.unwrap();
    let receipt = tx.confirm(&tx_hash, &provider).await.unwrap();
    assert_ne!(receipt.block_number.unwrap(), U64::from(0));
    assert_ne!(receipt.status.unwrap(), U64::from(0));
    assert_eq!(receipt.transaction_hash, tx_hash);
    let after = provider.get_balance(to, None).await.unwrap();
    assert_eq!(before + value, after);

    drop(anvil);
}
