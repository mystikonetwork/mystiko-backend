use mystiko_server_utils::token_price::PriceMiddlewareError;

#[tokio::test]
async fn test_error() {
    let err = PriceMiddlewareError::FileError("test".to_string());
    let err_str = format!("{:?}", err);
    assert_eq!(err_str, "FileError(\"test\")");
}
