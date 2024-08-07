use crate::{
    errors::{BiAnApiError, BiAnResult},
    KLineInterval, WS_BASE_URL,
};
use concat_string::concat_string;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::{
    net::TcpStream,
    sync::{mpsc, RwLock},
    task::{JoinHandle, JoinSet},
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

#[derive(Debug)]
pub enum ChannelPath {
    /// 订阅行情数据，
    /// 空的MarketPath表示暂时不订阅，而是等待以后手动添加订阅
    MarketPath(HashSet<String>),
    /// 订阅账户数据
    ListenKey(String),
}

impl ChannelPath {
    /// 对于行情类订阅，返回：空字符串或者"<StreamName1>/<StreamName2>..."
    /// 对于账户类定于，返回："<ListenKey>"
    fn to_path(&self) -> String {
        match self {
            Self::MarketPath(s) => Vec::from_iter(s.iter().map(|x| x.as_str())).join("/"),
            Self::ListenKey(l) => l.into(),
        }
    }

    /// 空的MarketPath表示暂时不订阅，而是等待以后手动添加订阅
    /// ```
    /// ChannelPath::market_path(HashSet::from(["btcusdt@aggTrade".to_string()]));
    /// ```
    pub fn market_path(content: HashSet<String>) -> Self {
        Self::MarketPath(content)
    }

    /// 生成空的MarketPath，空的MarketPath表示暂时不订阅，而是等待以后手动添加订阅
    pub fn empty_market_path() -> Self {
        Self::MarketPath(HashSet::default())
    }

    /// 订阅行情数据，
    /// ```
    /// ChannelPath::listen_key_path("djkas812klkadjkflaslkdfasd".to_string());
    /// ```
    fn listen_key_path(listen_key: String) -> Self {
        Self::ListenKey(listen_key)
    }

    /// 向MarketPath中添加数据，如果是ListenKey类型，则不做任何事情
    /// 用于手动订阅时添加指定要订阅的channel
    fn extend(&mut self, contents: HashSet<String>) {
        match self {
            ChannelPath::MarketPath(h) => h.extend(contents),
            ChannelPath::ListenKey(_) => {}
        }
    }

    /// 从MarketPath中移除数据，如果是ListenKey类型，则不做任何事情
    /// 用于手动取消订阅时移除指定要取消订阅的channel
    fn remove(&mut self, data: &str) {
        match self {
            ChannelPath::MarketPath(h) => {
                h.remove(data);
            }
            ChannelPath::ListenKey(_) => {}
        }
    }
}

/// 内部ws连接，只支持订阅组合Stream(参考<https://binance-docs.github.io/apidocs/spot/cn/#websocket>)
#[derive(Debug)]
struct WS {
    /// WebSocketStream
    ws_writer: RwLock<WsSink>,
    ws_reader: RwLock<WsStream>,
    channel_path: RwLock<ChannelPath>,
}

impl WS {
    fn make_ws_url(channel_path: &ChannelPath) -> String {
        let path = channel_path.to_path();
        if !path.is_empty() {
            concat_string!(WS_BASE_URL, "/stream?streams=", path)
            // format!("{}/stream?streams={}", WS_BASE_URL, path)
        } else {
            concat_string!(WS_BASE_URL, "/stream", path)
            // format!("{}/stream", WS_BASE_URL)
        }
    }

    async fn channel_path(&self) -> String {
        self.channel_path.read().await.to_path()
    }

    /// 建立ws连接，并处理返回数据  
    /// 每次调用都只能订阅单个频道(channel参数)  
    async fn new(channel_path: ChannelPath) -> BiAnResult<Self> {
        match &channel_path {
            ChannelPath::MarketPath(v) => {
                if v.len() > 1024 {
                    return Err(BiAnApiError::TooManySubscribes(v.len()));
                }
            }
            ChannelPath::ListenKey(_) => {}
        }

        let url = Self::make_ws_url(&channel_path);

        let (ws_stream, _response) = connect_async(&url).await?;
        let (ws_writer, ws_reader) = ws_stream.split();

        Ok(WS {
            ws_reader: RwLock::new(ws_reader),
            ws_writer: RwLock::new(ws_writer),
            channel_path: RwLock::new(channel_path),
        })
    }

    // fn tls_connector() {

    // }

    /// 创建一个新的WebSocketStream，并将其替换该ws连接内部已有的WebSocketStream  
    async fn replace_inner_stream(&self) {
        let dur = Duration::from_millis(500);
        let s = self.channel_path.read().await;
        let url = Self::make_ws_url(&s);
        loop {
            if let Ok((ws_stream, _response)) = connect_async(&url).await {
                let (ws_writer, ws_reader) = ws_stream.split();
                *self.ws_reader.write().await = ws_reader;
                *self.ws_writer.write().await = ws_writer;
                warn!(
                    "build websocket stream success: {}",
                    self.channel_path().await
                );
                break;
            }
            warn!(
                "retry to build websocket stream: {}",
                self.channel_path().await
            );
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
    /// 返回Some(close_reason)表示要重建ws，返回None表示一切正常无需重建
    async fn handle_msg(&self, msg: Message, data_sender: &mpsc::Sender<String>) -> Option<String> {
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
                    error!("ws closed: {}, <{}>", e, self.channel_path().await);
                }
            }
            Message::Close(Some(data)) => {
                return Some(data.reason.to_string());
            }
            _ => (),
        }
        None
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
            error!("channel closed: {}, <{}>", e, self.channel_path().await,);
        }
    }

    /// 手动订阅
    async fn subscribe(&self, channel_path: ChannelPath, id: u64) {
        let contents = match channel_path {
            ChannelPath::MarketPath(s) => s,
            ChannelPath::ListenKey(_) => panic!("subscribe account data not allowed"),
        };

        let json_text = serde_json::json!({
            "method": "SUBSCRIBE",
            "params": contents,
            "id": id
        });
        let msg = Message::Text(json_text.to_string());
        match self.ws_writer.write().await.send(msg).await {
            Ok(_) => {
                self.channel_path.write().await.extend(contents);
            }
            Err(e) => {
                error!("channel closed: {}, <{}>", e, self.channel_path().await,);
            }
        }
    }

    /// 取消订阅
    async fn unsubscribe(&self, channel_path: ChannelPath, id: u64) {
        let contents = match channel_path {
            ChannelPath::MarketPath(s) => s,
            ChannelPath::ListenKey(_) => panic!("unsubscribe account data not allowed"),
        };
        let json_text = serde_json::json!({
            "method": "UNSUBSCRIBE",
            "params": contents,
            "id": id
        });
        let msg = Message::Text(json_text.to_string());
        match self.ws_writer.write().await.send(msg).await {
            Ok(_) => {
                let mut inner = self.channel_path.write().await;
                for c in contents {
                    inner.remove(&c);
                }
            }
            Err(e) => {
                error!("channel closed: {}, <{}>", e, self.channel_path().await,);
            }
        }
    }
}

/// ws连接，只支持订阅组合Stream，以及手动订阅和取消订阅
///
/// (参考<https://binance-docs.github.io/apidocs/spot/cn/#websocket>)
#[derive(Debug, Clone)]
#[must_use = "`WsClient` must be use"]
pub struct WsClient {
    // 不使用DashMap，因为它在不切换任务的过程中重复get_mut时，不会释放锁，
    // 相邻两次之间必须至少有一个额外的异步任务才可以，比如`tokio::task::yield_now().await`
    // pub ws: Arc<DashMap<(), WS>>,
    ws: Arc<WS>,
    close_sender: mpsc::Sender<bool>,
}

impl WsClient {
    /// 新建到币安的ws连接客户端，该操作不会阻塞
    ///
    /// 可通过返回的JoinHandle来等待或终止后台的异步任务
    ///
    /// ```
    /// let (data_tx, mut data_rx) = tokio::sync::mpsc::Channel::<String>(1000);
    /// let channel_path = ChannelPath::market_path(HashSet::from(["btcusdt@aggTrade".to_string()]));
    /// // 通过task.abort()，可中断ws_client内部运行的异步任务
    /// let (ws_client, task) = WsClient::new(channel_path, data_tx).await.unwrap();
    ///
    /// let close_sender = ws_client.close_sender().await;
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
    ///     ws_client.close(true).await;
    /// });
    /// ```
    pub async fn new(
        channel_path: ChannelPath,
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        let ws = WS::new(channel_path).await?;
        let (close_sender, close_receiver) = mpsc::channel::<bool>(1);
        let s = Self {
            ws: Arc::new(ws),
            close_sender,
        };

        let task = {
            let mut join_set = JoinSet::new();

            // 接收ws通道数据的后台任务
            let ss = s.clone();
            join_set.spawn(async move {
                ss.read_from_channel(data_sender).await;
            });

            // 接收关闭客户端信号的后台任务
            let ss = s.clone();
            join_set.spawn(async move {
                ss.read_close_channel(close_receiver).await;
            });

            // 监控后台任务的任务
            let ss = s.clone();
            tokio::spawn(async move {
                let _ = join_set.join_next().await;
                join_set.shutdown().await;
                let _ = ss.ws.close_stream().await;
                error!("ws_client terminated");
            })
        };

        Ok((s, task))
    }

    /// 手动订阅
    /// ```
    /// let channel_path = ChannelPath::market_path(HashSet::from(["btcusdt@aggTrade".to_string()]));
    /// self.subscribe(channel_path).await;
    /// ```
    pub async fn subscribe(&self, channel_path: ChannelPath, id: u64) {
        self.ws.subscribe(channel_path, id).await;
    }

    /// 取消订阅
    /// ```
    /// let channel_path = ChannelPath::market_path(HashSet::from(["btcusdt@aggTrade".to_string()]));
    /// self.unsubscribe(channel_path).await;
    /// ```
    pub async fn unsubscribe(&self, channel_path: ChannelPath, id: u64) {
        self.ws.unsubscribe(channel_path, id).await;
    }

    /// 列出订阅内容，可用于检查是否订阅成功.
    /// 向通道发送信息，查看订阅结果，通道会响应id和result字段
    /// id参数随意，响应中的id字段总是和该给定的id相同
    pub async fn list_subscribers(&self, id: u64) {
        self.ws.list_sub(id).await;
    }

    /// 强制或不强制关闭ws client
    ///
    /// force: false表示重建内部ws连接，true表示强制关闭WsClient
    pub async fn close(&self, force: bool) {
        if self.close_sender.send(force).await.is_err() {
            error!("close receiver of close client already closed");
        }
    }

    /// 读取数据通道，当无法重建ws时才返回
    async fn read_from_channel(&self, data_sender: mpsc::Sender<String>) {
        //@ 循环不断地接收ws的信息
        loop {
            let err_msg = {
                let mut ws_reader = self.ws.ws_reader.write().await;
                loop {
                    if data_sender.is_closed() {
                        return;
                    }
                    let msg = ws_reader.next().await;
                    if let Some(msg) = msg {
                        match msg {
                            Ok(msg) => {
                                if let Some(err_msg) = self.ws.handle_msg(msg, &data_sender).await {
                                    break err_msg;
                                }
                            }
                            Err(e) => {
                                break e.to_string();
                            }
                        }
                    }
                }
            };
            warn!("ws closed: {}, {}", err_msg, self.ws.channel_path().await);
            self.ws.replace_inner_stream().await;
        }
    }

    /// 读取关闭WsClient的信号
    async fn read_close_channel(&self, mut close_receiver: mpsc::Receiver<bool>) {
        let dur = tokio::time::Duration::from_millis(100);
        loop {
            match close_receiver.recv().await {
                Some(data) => {
                    // 关闭失败不退出，因为连接可能会被重建
                    if let Err(e) = self.ws.close_stream().await {
                        error!("close stream error: {}", e);
                    }
                    if data {
                        return;
                    }
                }
                None => {
                    warn!("ws close channel closed");
                    tokio::time::sleep(dur).await;
                }
            }
        }
    }

    /// 以ws_client的方式订阅"归集交易流"(该操作不会"阻塞"当前异步任务)  
    /// 归集交易 stream 推送交易信息，是对单一订单的集合  
    /// symbols参数忽略大小写  
    pub async fn agg_trade(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <sym>@aggTrade
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@aggTrade"))
            .collect::<HashSet<String>>();
        Self::new(ChannelPath::market_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"逐笔交易流"(该操作不会"阻塞"当前异步任务)  
    /// 逐笔交易推送每一笔成交的信息。成交，或者说交易的定义是仅有一个吃单者与一个挂单者相互交易  
    /// symbols参数忽略大小写  
    pub async fn trade(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <sym>@trade
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@trade"))
            .collect::<HashSet<String>>();
        Self::new(ChannelPath::market_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"K线数据流"(该操作不会"阻塞"当前异步任务)  
    /// K线stream逐秒推送所请求的K线种类(最新一根K线)的更新  
    /// symbols参数忽略大小写  
    pub async fn kline(
        interval: &str,
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        if !KLineInterval::is_intv(interval) {
            panic!("argument error: <{}> invalid kline interval", interval);
        }

        // channel: kline_<interval>
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@kline_", interval))
            .collect::<HashSet<String>>();

        Self::new(ChannelPath::market_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"按symbol的精简Ticker"(该操作不会"阻塞"当前异步任务)  
    /// 按Symbol刷新的最近24小时精简ticker信息  
    /// symbols参数忽略大小写  
    pub async fn mini_ticker(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <sym>@miniTicker
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@miniTicker"))
            .collect::<HashSet<String>>();
        Self::new(ChannelPath::market_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"全市场所有Symbol的精简Ticker"(该操作不会"阻塞"当前异步任务)  
    /// 推送所有交易对的最近24小时精简ticker信息.需注意，只有更新的ticker才会被推送
    pub async fn all_mini_ticker(
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: !miniTicker@arr
        let channel_path = HashSet::from(["!miniTicker@arr".into()]);
        Self::new(ChannelPath::market_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"按symbol的完整Ticker"(该操作不会"阻塞"当前异步任务)  
    /// 每秒推送单个交易对的过去24小时滚动窗口标签统计信息  
    /// symbols参数忽略大小写  
    pub async fn ticker(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <sym>@ticker
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@ticker"))
            .collect::<HashSet<String>>();
        Self::new(ChannelPath::market_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"全市场所有Symbol的完整Ticker"(该操作不会"阻塞"当前异步任务)  
    /// 推送所有交易对的最近24小时完整ticker信息.需注意，只有更新的ticker才会被推送
    pub async fn all_ticker(
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: !ticker@arr
        let channel_path = HashSet::from(["!ticker@arr".into()]);
        Self::new(ChannelPath::market_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"按Symbol的最优挂单信息"(该操作不会"阻塞"当前异步任务)  
    /// 实时推送指定交易对最优挂单信息  
    /// symbols参数忽略大小写  
    pub async fn bookticker(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <sym>@bookTicker
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@bookTicker"))
            .collect::<HashSet<String>>();
        Self::new(ChannelPath::market_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"有限档深度信息"(该操作不会"阻塞"当前异步任务)  
    /// 每100毫秒推送有限档深度信息。level表示几档买卖单信息, 可选5/10/20档，
    /// 档数表示返回结果中包含几个挂单信息，5档表示返回最新的5个买盘和5个卖盘挂单信息，  
    /// symbols参数忽略大小写  
    pub async fn depth_with_level(
        symbols: Vec<String>,
        level: u8,
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <symbol>@depth<levels> 或 <symbol>@depth<levels>@100ms

        if ![5u8, 10u8, 20u8].contains(&level) {
            panic!("argument error: <{}> level is invalid", level)
        }

        let channel_path = symbols
            .iter()
            .map(|sym| {
                concat_string!(
                    sym.to_ascii_lowercase(),
                    "@depth",
                    level.to_string(),
                    "@100ms"
                )
            })
            // .map(|sym| format!("{}@depth{}@100ms", sym.to_lowercase(), level))
            .collect::<HashSet<String>>();

        Self::new(ChannelPath::market_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"增量深度信息"(该操作不会"阻塞"当前异步任务)  
    /// 每100毫秒推送orderbook的变化部分(如果有)  
    /// symbols参数忽略大小写  
    pub async fn depth_incr(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <symbol>@depth 或 <symbol>@depth@100ms
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@@depth@100ms"))
            .collect::<HashSet<String>>();

        Self::new(ChannelPath::market_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"Websocket账户信息推送"(该操作不会"阻塞"当前异步任务)  
    ///
    /// 包括账户更新、余额更新、订单更新，参考(<https://binance-docs.github.io/apidocs/spot/cn/#websocket-2>)  
    pub async fn account(
        listen_key: String,
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        Self::new(ChannelPath::listen_key_path(listen_key), data_sender).await
    }
}
