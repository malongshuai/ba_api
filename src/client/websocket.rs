use crate::{
    errors::{BiAnApiError, BiAnResult},
    KLineInterval, WS_BASE_URL,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{sync::Arc, time::Duration};
use tokio::{
    net::TcpStream,
    sync::{mpsc, RwLock},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
    MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, warn};

type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// 内部ws连接，只支持订阅组合Stream(参考<https://binance-docs.github.io/apidocs/spot/cn/#websocket>)
#[derive(Debug)]
pub struct WS {
    /// WebSocketStream
    pub ws_writer: RwLock<WsSink>,
    pub ws_reader: RwLock<WsStream>,
    /// 该ws订阅的频道
    pub names: Vec<String>,
    /// 该ws连接订阅的频道名称
    pub channel: String,
    /// 该ws的URL
    pub url: String,
}

impl WS {
    fn make_ws_url(channel: &str, name: &[String], base_url: &str) -> String {
        let streams = if name.is_empty() {
            channel.to_string()
        } else {
            let metadata = name
                .iter()
                .map(|sym| format!("{}@{}", sym, channel))
                .collect::<Vec<String>>();
            metadata.join("/")
        };

        format!("{}/stream?streams={}", base_url, streams)
    }

    /// 建立ws连接，并处理返回数据  
    /// 每次调用都只能订阅单个频道(channel参数)  
    pub async fn new(channel: &str, names: Vec<String>) -> BiAnResult<WS> {
        if names.len() > 1024 {
            return Err(BiAnApiError::TooManySubscribes(names.len()));
        }

        let base_url = WS_BASE_URL;
        let names: Vec<String> = names.iter().map(|x| x.to_string()).collect();
        let url = Self::make_ws_url(channel, &names, base_url);

        let (ws_stream, _response) = connect_async(&url).await?;
        let (ws_writer, ws_reader) = ws_stream.split();

        Ok(WS {
            ws_reader: RwLock::new(ws_reader),
            ws_writer: RwLock::new(ws_writer),
            names,
            channel: channel.to_string(),
            url,
        })
    }

    /// 创建一个新的WebSocketStream，并将其替换该ws连接内部已有的WebSocketStream  
    pub async fn replace_inner_stream(&self) {
        let dur = Duration::from_millis(500);
        loop {
            if let Ok((ws_stream, _response)) = connect_async(&self.url).await {
                let (ws_writer, ws_reader) = ws_stream.split();
                *self.ws_reader.write().await = ws_reader;
                *self.ws_writer.write().await = ws_writer;
                warn!("build websocket stream success: {}", self.url);
                break;
            }
            warn!("retry to build websocket stream: {}", self.url);
            tokio::time::sleep(dur).await;
        }
    }

    /// 关闭ws conn_stream
    async fn close_stream(&self) -> BiAnResult<()> {
        let close_frame = CloseFrame {
            code: CloseCode::Normal,
            reason: std::borrow::Cow::Borrowed("force close"),
        };
        let msg = Message::Close(Some(close_frame));
        self.ws_writer.write().await.send(msg).await?;
        Ok(())
    }

    /// 处理ws接收到的信息，并且在接收到ws关闭信息的时候替换重建ws
    async fn handle_msg(&self, msg: Message, data_sender: mpsc::Sender<String>) {
        match msg {
            Message::Text(data) => {
                if data_sender.send(data).await.is_err() {
                    error!("Data Receiver already closed");
                }
            }
            Message::Ping(d) => {
                debug!("received Ping Frame");
                let msg = Message::Pong(d);
                if let Err(e) = self.ws_writer.write().await.send(msg).await {
                    error!("channel({}) closed: {}", self.channel, e);
                }
            }
            Message::Close(Some(data)) => {
                warn!(
                    "websocket({}) close reason: <{}>, <{}>",
                    self.channel,
                    data.reason,
                    self.names.join(",")
                );
                self.replace_inner_stream().await;
            }
            _ => (),
        }
    }

    /// 列出订阅内容，可用于检查是否订阅成功
    /// 向通道发送信息，查看订阅结果，通道会响应id和result字段
    /// id参数随意，响应中的id字段总是和该给定的id相同以示对应
    async fn list_sub(&self, id: u64) {
        let json_text = serde_json::json!({
          "method": "LIST_SUBSCRIPTIONS",
          "id": id
        });
        let msg = Message::Text(json_text.to_string());
        if let Err(e) = self.ws_writer.write().await.send(msg).await {
            error!("channel({}) closed: {}", self.channel, e);
        }
    }
}

/// ws连接，只支持订阅组合Stream(参考<https://binance-docs.github.io/apidocs/spot/cn/#websocket>)
/// ```rust
/// let (data_tx, mut data_rx) = mpsc::channel::<String>(1000);
/// let (close_sender, mut close_receiver) = mpsc::channel::<bool>(1);
/// let mut wsc= WsClient::new("kline_1m", vec!["btcusdt", "ethusdt"], close_receiver).await.unwrap();
///
/// // 接收数据
/// tokio::spawn(async move {
///     while let Some(x) = data_rx.recv().await {
///         println!("channel received: {}", x);
///     }
/// });
///
/// // 20秒后强制关闭ws_client
/// tokio::spawn(async move {
///     tokio::time::sleep(Duration::from_secs(20)).await;
///     WsClient::close_client(close_sender.clone(), true).await;
///     debug!("send close");
/// });
///
/// wsc.ws_client(data_tx).await.unwrap();
/// ```
#[derive(Debug, Clone)]
#[must_use = "`WsClient` must be use"]
pub struct WsClient {
    // 不使用DashMap，因为它在不切换任务的过程中重复get_mut时，不会释放锁，
    // 相邻两次之间必须至少有一个额外的异步任务才可以，比如`tokio::task::yield_now().await`
    // pub ws: Arc<DashMap<(), WS>>,
    pub ws: Arc<WS>,
    close_sender: Arc<RwLock<mpsc::Sender<bool>>>,
    close_receiver: Arc<RwLock<mpsc::Receiver<bool>>>,
}

impl WsClient {
    /// 新建到币安的ws连接客户端，需传入一个mpsc::Receiver来接收连接关闭的通知
    pub async fn new(channel: &str, names: Vec<String>) -> BiAnResult<Self> {
        let ws = WS::new(channel, names).await?;
        let (close_sender, close_receiver) = mpsc::channel::<bool>(1);
        Ok(Self {
            ws: Arc::new(ws),
            close_receiver: Arc::new(RwLock::new(close_receiver)),
            close_sender: Arc::new(RwLock::new(close_sender)),
        })
    }

    /// 列出订阅内容，可用于检查是否订阅成功.
    /// 向通道发送信息，查看订阅结果，通道会响应id和result字段
    /// id参数随意，响应中的id字段总是和该给定的id相同
    pub async fn list_subscribers(&self, id: u64) {
        self.ws.list_sub(id).await;
    }

    /// 获取
    pub async fn close_sender(&self) -> mpsc::Sender<bool> {
        self.close_sender.read().await.clone()
    }

    /// 需结合ws_client使用，强制或不强制关闭ws client
    /// ```rust
    /// let (close_sender, close_rx) = mpsc::channel::<bool>(1);
    /// let mut wsc = WsClient::new("kline_1m", vec!["btcusdt", "ethusdt"], close_rx).await.unwrap();
    ///
    /// // 不强制关闭ws，只是关闭到币安的连接，但会自动重建连接
    /// WsClient::close_client(close_sender.clone(), false);
    /// // 不强制关闭ws，只是关闭到币安的连接，不会再重建连接
    /// WsClient::close_client(close_sender.clone(), true);
    /// ```
    pub async fn close_client(close_sender: mpsc::Sender<bool>, force: bool) {
        if close_sender.send(force).await.is_err() {
            error!("close receiver of close client already closed");
        }
    }

    /// 建立一个断连后会自动重连的ws客户端(会阻塞当前异步任务)  
    /// 该ws客户端接收到的数据(String)，都将通过mpsc的sender发送出去  
    /// 如果需要强制关闭ws_client，使用close_client发送true参数，如果需要手动断开但自动重连(相当于重建连接)，使用close_client发送false参数  
    /// ```rust
    /// let (data_tx, mut data_rx) = mpsc::channel::<String>(1000);
    /// let mut wsc = WsClient::new("kline_1m", vec!["btcusdt", "ethusdt"]).await.unwrap();
    /// let close_sender = wsc.close_sender().await;
    ///
    /// // 接收数据
    /// tokio::spawn(async move {
    ///     while let Some(x) = data_rx.recv().await {
    ///         println!("channel received: {}", x);
    ///     }
    /// });
    ///
    /// // 20秒后强制关闭ws_client
    /// tokio::spawn(async move {
    ///     tokio::time::sleep(Duration::from_secs(20)).await;
    ///     WsClient::close_client(close_sender.clone(), true).await;
    ///     debug!("send close");
    /// });
    ///
    /// wsc.sub_channel(data_tx).await.unwrap();
    /// ```
    pub async fn sub_channel(&self, data_sender: mpsc::Sender<String>) -> BiAnResult<()> {
        let ws_self = self.clone();
        //@ 循环不断地接收ws的信息，当无法重建ws时才返回
        let mut msg_handle_task = tokio::spawn(async move {
            loop {
                if data_sender.is_closed() {
                    break;
                }
                let ws = ws_self.ws.clone();
                let msg = ws.ws_reader.write().await.next().await;
                if let Some(msg) = msg {
                    match msg {
                        Ok(msg) => {
                            ws.handle_msg(msg, data_sender.clone()).await;
                        }
                        Err(e) => {
                            warn!("ws closed({}): {}, {}", ws.channel, ws.url, e);
                            ws.replace_inner_stream().await;
                        }
                    }
                }
            }
        });
        let ws_self = self.clone();
        let mut close_handle_task = tokio::spawn(async move {
            loop {
                match ws_self.close_receiver.write().await.recv().await {
                    Some(data) => {
                        // 关闭失败不退出，因为连接可能会被重建
                        if let Err(e) = ws_self.ws.close_stream().await {
                            error!("close stream error: {}", e);
                        }
                        if data {
                            break;
                        }
                    }
                    None => {
                        warn!("ws close channel closed");
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });
        if let Err(e) = tokio::try_join!(&mut msg_handle_task, &mut close_handle_task) {
            msg_handle_task.abort();
            close_handle_task.abort();
            error!("ws_client break out: {}", e);
        }
        Ok(())
    }

    /// 以ws_client的方式订阅"归集交易流"(将"阻塞"当前异步任务)  
    /// 归集交易 stream 推送交易信息，是对单一订单的集合  
    /// symbols参数忽略大小写  
    pub async fn agg_trade(symbols: Vec<String>) -> BiAnResult<Self> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Self::new("aggTrade", symbols).await
    }

    /// 以ws_client的方式订阅"逐笔交易流"(将"阻塞"当前异步任务)  
    /// 逐笔交易推送每一笔成交的信息。成交，或者说交易的定义是仅有一个吃单者与一个挂单者相互交易  
    /// symbols参数忽略大小写  
    pub async fn trade(symbols: Vec<String>) -> BiAnResult<Self> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Self::new("trade", symbols).await
    }

    /// 以ws_client的方式订阅"K线数据流"(将"阻塞"当前异步任务)  
    /// K线stream逐秒推送所请求的K线种类(最新一根K线)的更新  
    /// symbols参数忽略大小写  
    pub async fn kline(interval: &str, symbols: Vec<String>) -> BiAnResult<Self> {
        if !KLineInterval::is_intv(interval) {
            panic!("argument error: <{}> invalid kline interval", interval);
        }
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        let channel = format!("kline_{}", interval);
        Self::new(&channel, symbols).await
    }

    /// 以ws_client的方式订阅"按symbol的精简Ticker"(将"阻塞"当前异步任务)  
    /// 按Symbol刷新的最近24小时精简ticker信息  
    /// symbols参数忽略大小写  
    pub async fn mini_ticker(symbols: Vec<String>) -> BiAnResult<Self> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Self::new("miniTicker", symbols).await
    }

    /// 以ws_client的方式订阅"全市场所有Symbol的精简Ticker"(将"阻塞"当前异步任务)  
    /// 推送所有交易对的最近24小时精简ticker信息.需注意，只有更新的ticker才会被推送
    pub async fn all_mini_ticker() -> BiAnResult<Self> {
        Self::new("arr", vec!["!miniTicker".to_string()]).await
    }

    /// 以ws_client的方式订阅"按symbol的完整Ticker"(将"阻塞"当前异步任务)  
    /// 每秒推送单个交易对的过去24小时滚动窗口标签统计信息  
    /// symbols参数忽略大小写  
    pub async fn ticker(symbols: Vec<String>) -> BiAnResult<Self> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Self::new("ticker", symbols).await
    }

    /// 以ws_client的方式订阅"全市场所有Symbol的完整Ticker"(将"阻塞"当前异步任务)  
    /// 推送所有交易对的最近24小时完整ticker信息.需注意，只有更新的ticker才会被推送
    pub async fn all_ticker() -> BiAnResult<Self> {
        Self::new("arr", vec!["!ticker".to_string()]).await
    }

    /// 以ws_client的方式订阅"按Symbol的最优挂单信息"(将"阻塞"当前异步任务)  
    /// 实时推送指定交易对最优挂单信息  
    /// symbols参数忽略大小写  
    pub async fn bookticker(symbols: Vec<String>) -> BiAnResult<Self> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Self::new("bookTicker", symbols).await
    }

    // 2022-11月将下线 !bookTicker 的订阅通道
    // /// 以ws_client的方式订阅"全市场最优挂单信息"(将"阻塞"当前异步任务)
    // /// 实时推送所有交易对最优挂单信息
    // pub async fn all_bookticker() -> BiAnResult<Self> {
    //     let names: Vec<String> = vec![];
    //     Self::new("!bookTicker", names).await
    // }

    /// 以ws_client的方式订阅"有限档深度信息"(将"阻塞"当前异步任务)  
    /// 每100毫秒推送有限档深度信息。level表示几档买卖单信息, 可选5/10/20档，
    /// 档数表示返回结果中包含几个挂单信息，5档表示返回最新的5个买盘和5个卖盘挂单信息，  
    /// symbols参数忽略大小写  
    pub async fn depth_with_level(symbols: Vec<String>, level: u8) -> BiAnResult<Self> {
        if ![5u8, 10u8, 20u8].contains(&level) {
            panic!("argument error: <{}> level is invalid", level)
        }
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        let channel = format!("depth{}@100ms", level);
        Self::new(&channel, symbols).await
    }

    /// 以ws_client的方式订阅"增量深度信息"(将"阻塞"当前异步任务)  
    /// 每100毫秒推送orderbook的变化部分(如果有)  
    /// symbols参数忽略大小写  
    pub async fn depth_incr(symbols: Vec<String>) -> BiAnResult<Self> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Self::new("depth@100ms", symbols).await
    }

    /// 以ws_client的方式订阅"Websocket账户信息推送"(将"阻塞"当前异步任务)  
    ///
    /// 包括账户更新、余额更新、订单更新，参考(<https://binance-docs.github.io/apidocs/spot/cn/#websocket-2>)  
    pub async fn account(listen_key: String) -> BiAnResult<Self> {
        Self::new(&listen_key, vec![]).await
    }
}
