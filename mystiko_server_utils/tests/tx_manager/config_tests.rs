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
    assert_eq!(cfg.get_max_confirm_count(1), 83);
    assert_eq!(cfg.gas_limit_reserve_percentage, 10);

    let cfg = TxManagerConfig::new(Some(PathBuf::from("tests/tx_manager/empty"))).unwrap();
    let cfg1 = cfg.chain_config(&1).unwrap();
    assert_eq!(cfg1.get_max_confirm_count(1), 83);
    let cfg137 = cfg.chain_config(&137).unwrap();
    assert_eq!(cfg137.min_priority_fee_per_gas, None);
    let cfg56 = cfg.chain_config(&56).unwrap();
    assert!(cfg56.get_force_gas_price(56));
    let cfg4002 = cfg.chain_config(&4002).unwrap();
    assert!(cfg4002.get_force_gas_price(4002));

    let cfg = TxManagerConfig::new(Some(PathBuf::from("tests/tx_manager/files"))).unwrap();
    let cfg1 = cfg.chain_config(&1).unwrap();
    assert_eq!(cfg1.get_max_confirm_count(1), 123456);
    let cfg137 = cfg.chain_config(&137).unwrap();
    assert_eq!(cfg137.min_priority_fee_per_gas, None);
    let cfg250 = cfg.chain_config(&250).unwrap();
    assert!(cfg250.get_force_gas_price(250));
    let cfg4002 = cfg.chain_config(&4002).unwrap();
    assert!(cfg4002.get_force_gas_price(4002));

    env::set_var("MYSTIKO_TX_MANAGER.CHAINS.1.MAX_CONFIRM_COUNT", "112");
    env::set_var("MYSTIKO_TX_MANAGER.CHAINS.1.GAS_LIMIT_RESERVE_PERCENTAGE", "24");
    let cfg = TxManagerConfig::new(None).unwrap();
    let cfg1 = cfg.chain_config(&1).unwrap();
    assert_eq!(cfg1.get_max_confirm_count(1), 112);
    assert_eq!(cfg1.gas_limit_reserve_percentage, 24);
    let cfg137 = cfg.chain_config(&137).unwrap();
    assert_eq!(cfg137.min_priority_fee_per_gas, None);
    let cfg250 = cfg.chain_config(&250).unwrap();
    assert!(cfg250.get_force_gas_price(250));
    let cfg56 = cfg.chain_config(&56).unwrap();
    assert!(cfg56.get_force_gas_price(56));

    env::remove_var("MYSTIKO_TX_MANAGER.CHAINS.1.MAX_CONFIRM_COUNT");
    env::remove_var("MYSTIKO_TX_MANAGER.CHAINS.1.GAS_LIMIT_RESERVE_PERCENTAGE");

    env::set_var("MYSTIKO_TX_MANAGER.CHAINS.1.MIN_PRIORITY_FEE_PER_GAS", "3");
    env::set_var("MYSTIKO_TX_MANAGER.CHAINS.1.MAX_PRIORITY_FEE_PER_GAS", "2");
    let cfg = TxManagerConfig::new(None).unwrap();
    let chain_cfg = cfg.chain_config(&1_u64);
    assert!(chain_cfg
        .err()
        .unwrap()
        .to_string()
        .contains("max_priority_fee_per_gas must be greater than min_priority_fee_per_gas"));
    env::remove_var("MYSTIKO_TX_MANAGER.CHAINS.1.MIN_PRIORITY_FEE_PER_GAS");
    env::remove_var("MYSTIKO_TX_MANAGER.CHAINS.1.MAX_PRIORITY_FEE_PER_GAS");

    env::set_var("MYSTIKO_TX_MANAGER.CHAINS.1.lower_gas_price_percentage", "86");
    let cfg = TxManagerConfig::new(None).unwrap();
    let chain_cfg = cfg.chain_config(&1_u64);
    assert!(chain_cfg
        .err()
        .unwrap()
        .to_string()
        .contains("lower_gas_price_percentage must be less than"));
    env::remove_var("MYSTIKO_TX_MANAGER.CHAINS.1.lower_gas_price_percentage");
}
