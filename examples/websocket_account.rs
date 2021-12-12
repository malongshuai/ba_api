use ba_api::client::RestConn;
use ba_api::client::WsClient;
use ba_api::WsResponse;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::debug;

// 替换为你自己的API KEY和Secret Key
const API_KEY: &str = "abcdefdjklfasjfklasdjflas";
const SEC_KEY: &str = "Mfnasldfjaklsjdfhsakjdfha";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    websocket().await;
}

#[allow(dead_code)]
async fn websocket() {
    // 创建http连接
    let rest_conn = RestConn::new(API_KEY, SEC_KEY, Some("http://127.0.0.1:8118"));

    // 生成一个ListenKey以便使用websocket订阅账户更新信息
    let listen_key = rest_conn.new_spot_listen_key().await.unwrap();

    // 创建mpsc通道用来接收websocket推送的数据，接收到的数据都是String类型
    let (data_sender, mut data_receiver) = mpsc::channel::<String>(1000);

    // 创建mpsc通道，用来传输是否要关闭websocket连接的通知
    let (close_sender, close_receiver) = mpsc::channel::<bool>(1);

    // 新生成一个任务使用data_receiver不断接收websocket推送的数据，并将其反序列化为账户信息
    tokio::spawn(async move {
        while let Some(x) = data_receiver.recv().await {
            let data = serde_json::from_str::<WsResponse>(&x);
            println!("channel received: {:?}", data);
        }
    });

    // 新生成一个任务使用close_sender来发送关闭websocket连接的通知，发送true表示强制关闭，不再自动重建连接，发送false表示关闭连接但会自动重连
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(600)).await;
        WsClient::close_client(close_sender, true).await;
        debug!("send close");
    });

    // 订阅币安的账户更新通道，币安推送的余额更新、订单更新信息等都将会接收到，并通过data_sender发送出去(发送的是String类型)
    WsClient::account(listen_key, data_sender, close_receiver)
        .await
        .unwrap();
}
