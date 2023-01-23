use ba_api::client::RestConn;
use tracing::debug;

fn api_key() -> Option<String> {
    std::env::var("BA_API_KEY").ok()
}

fn sec_key() -> Option<String> {
    std::env::var("BA_SEC_KEY").ok()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // 创建http连接
    let rest_conn = RestConn::new(
        api_key().unwrap(),
        sec_key().unwrap(),
        Some("http://127.0.0.1:8118".to_string()),
    )
    .await;

    // 获取BTCUSDT的最近5根K线
    let x = rest_conn
        .klines("BTCUSDT", "1m", None, None, Some(5u16))
        .await;
    debug!("{:?}", x);
}
