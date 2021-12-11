use std::time::Duration;

use ba_api::client::websocket::WsClient;
#[allow(unused_imports)]
use ba_api::client::RestConn;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use tokio::sync::mpsc;
#[allow(unused_imports)]
use tracing::debug;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let _api_key = "TAfI8Pqy66hJqOYiegSoijSy2SMp6qbWvoPZDidiLdF39mAodwfsKXQfRWmYBEhk";
    let _sec_key = "MOIeI2mK94Y513IEyrLHfCwh6M7RP8hhT719zgxWLwD5GKvO489Mlo2YdUP1tGM5";
    // let rest_conn = RestConn::new(_api_key, _sec_key, Some("http://127.0.0.1:8118"));
    // // let x = rest_conn.limit_order("CELRUSDT", "buy", 10.5, 0.06).await;
    // // debug!("{:?}", x);
    // // let x = rest_conn.limit_order("CELRUSDT", "buy", 10.5, 0.06).await;
    // // debug!("{:?}", x);
    // // let x = rest_conn.limit_order("CELRUSDT", "buy", 10.5, 0.06).await;
    // // debug!("{:?}", x);
    // let x = rest_conn.account().await;
    // debug!("{:?}", x);

    let (data_sender, mut data_receiver) = mpsc::channel::<String>(1000);
    let (close_sender, close_receiver) = mpsc::channel::<bool>(1);

    tokio::spawn(async move {
        while let Some(x) = data_receiver.recv().await {
            println!("channel received: {}", x);
        }
    });

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(20)).await;
        WsClient::close_client(close_sender, true).await;
        debug!("send close");
    });

    WsClient::kline(
        "1m",
        vec!["btcusdt", "ethusdt"],
        data_sender,
        close_receiver,
    )
    .await
    .unwrap();
}
