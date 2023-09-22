use mystiko_server_utils::tx_manager::config::TxManagerConfig;
use std::env;
use std::path::PathBuf;

#[tokio::test]
async fn test_read_config() {
    // test invalid config
    let cfg = TxManagerConfig::new(Some(PathBuf::from("tests/tx_manager/files/invalid")));
    assert!(cfg.is_err());

    let cfg = TxManagerConfig::new(None).unwrap();
    println!("cfg {}", serde_json::to_string(&cfg).unwrap());

    let cfg = cfg.chain_config(&1).unwrap();
    assert_eq!(cfg.max_confirm_count, 100);
    assert_eq!(cfg.gas_limit_reserve_percentage, 10);

    let cfg = TxManagerConfig::new(Some(PathBuf::from("tests/tx_manager/empty"))).unwrap();
    let cfg1 = cfg.chain_config(&1).unwrap();
    assert_eq!(cfg1.max_confirm_count, 100);
    let cfg137 = cfg.chain_config(&137).unwrap();
    assert_eq!(cfg137.min_priority_fee_per_gas, 30_000_000_000_u64);
    let cfg250 = cfg.chain_config(&250).unwrap();
    assert!(cfg250.force_gas_price);
    let cfg4002 = cfg.chain_config(&4002).unwrap();
    assert!(cfg4002.force_gas_price);

    let cfg = TxManagerConfig::new(Some(PathBuf::from("tests/tx_manager/files"))).unwrap();
    let cfg1 = cfg.chain_config(&1).unwrap();
    assert_eq!(cfg1.max_confirm_count, 123456);
    let cfg137 = cfg.chain_config(&137).unwrap();
    assert_eq!(cfg137.min_priority_fee_per_gas, 30_000_000_000_u64);
    let cfg250 = cfg.chain_config(&250).unwrap();
    assert!(cfg250.force_gas_price);
    let cfg4002 = cfg.chain_config(&4002).unwrap();
    assert!(cfg4002.force_gas_price);

    env::set_var("MYSTIKO_TX_MANAGER.CHAINS.1.MAX_CONFIRM_COUNT", "112");
    env::set_var("MYSTIKO_TX_MANAGER.CHAINS.1.GAS_LIMIT_RESERVE_PERCENTAGE", "24");
    let cfg = TxManagerConfig::new(None).unwrap();
    let cfg1 = cfg.chain_config(&1).unwrap();
    assert_eq!(cfg1.max_confirm_count, 112);
    assert_eq!(cfg1.gas_limit_reserve_percentage, 24);
    let cfg137 = cfg.chain_config(&137).unwrap();
    assert_eq!(cfg137.min_priority_fee_per_gas, 30000000000);
    let cfg250 = cfg.chain_config(&250).unwrap();
    assert!(cfg250.force_gas_price);
    let cfg4002 = cfg.chain_config(&4002).unwrap();
    assert!(cfg4002.force_gas_price);

    env::remove_var("MYSTIKO_TX_MANAGER.CHAINS.1.MAX_CONFIRM_COUNT");
    env::remove_var("MYSTIKO_TX_MANAGER.CHAINS.1.GAS_LIMIT_RESERVE_PERCENTAGE");
}
