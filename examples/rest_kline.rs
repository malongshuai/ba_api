use ba_api::client::RestConn;
use tracing::debug;

const API_KEY: &str = "dwfsKXQfRWmYBEhk";
const SEC_KEY: &str = "489Mlo2YdUP1tGM5";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // 创建http连接
    let rest_conn = RestConn::new(API_KEY, SEC_KEY, Some("http://127.0.0.1:8118"), None);

    // 获取BTCUSDT的最近5根K线
    let x = rest_conn
        .klines("BTCUSDT", "1m", None, None, Some(5u16))
        .await;
    debug!("{:?}", x);
}
