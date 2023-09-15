use mystiko_server_utils::tx_manager::TransactionMiddlewareError;

#[tokio::test]
async fn test_error() {
    let err = TransactionMiddlewareError::GasPriceError("test".to_string());
    let err_str = format!("{:?}", err);
    assert_eq!(err_str, "GasPriceError(\"test\")");
}
