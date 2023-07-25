use ba_api::client::RestConn;
// 需开启 websocket features
use ba_api::client::WsClient;
use ba_api::WsResponse;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::debug;
use tracing::info;

fn api_key() -> Option<String> {
    std::env::var("BA_API_KEY").ok()
}

fn sec_key() -> Option<String> {
    std::env::var("BA_SEC_KEY").ok()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let ws_client = websocket().await;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        ws_client.list_sub(1).await;
    }
}

#[allow(dead_code)]
async fn websocket() -> WsClient {
    // 创建http连接
    let rest_conn = RestConn::new(api_key().unwrap(), sec_key().unwrap(), None).await;

    // 生成一个ListenKey以便使用websocket订阅账户更新信息
    let listen_key = rest_conn.new_spot_listen_key().await.unwrap();
    info!("listen_key: {}", listen_key);

    // 创建mpsc通道用来接收websocket推送的数据，接收到的数据都是String类型
    let (data_sender, mut data_receiver) = mpsc::channel::<String>(1000);

    // 新生成一个任务使用data_receiver不断接收websocket推送的数据，并将其反序列化为账户信息
    tokio::spawn(async move {
        while let Some(x) = data_receiver.recv().await {
            // info!("channel received: {x}");
            let data = serde_json::from_str::<WsResponse>(&x);
            info!("channel received: {:?}", data);
        }
    });

    let ws_client = WsClient::account(listen_key).await.unwrap();
    let close_sender = ws_client.close_sender().await;

    // 新生成一个任务使用close_sender来发送关闭websocket连接的通知，发送true表示强制关闭，不再自动重建连接，发送false表示关闭连接但会自动重连
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(600)).await;
        WsClient::close_client(close_sender, true).await;
        debug!("send close");
    });

    // 订阅币安的账户更新通道，币安推送的余额更新、订单更新信息等都将会接收到，并通过data_sender发送出去(发送的是String类型)
    // ws_client.sub_channel(data_sender).await.unwrap();

    let wc = ws_client.clone();
    tokio::spawn(async move {
        wc.sub_channel(data_sender).await.unwrap();
    });

    ws_client
}
