use super::params::{PKLine, PSessionLogon, PSessionStatus, PUserDataStream, PWebSocketApi, Param};
use crate::{
    ApiSecKey, KLineInterval, WebsocketApiResponse, WsResponse,
    errors::{BiAnApiError, BiAnResult},
};
use ba_global::{WS_API_URL, WS_STREAM_URL};
use concat_string::concat_string;
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::Serialize;
use std::{
    collections::HashSet,
    sync::{
        Arc,
        atomic::{self, AtomicBool},
    },
    time::Duration,
};
use tokio::{
    net::TcpStream,
    sync::{RwLock, mpsc},
    task::{JoinHandle, JoinSet},
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{
        Message, Utf8Bytes,
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Debug)]
pub enum ChannelPath {
    /// 订阅行情数据流，
    /// 空的MarketPath表示暂时不订阅，而是等待以后手动添加订阅
    MarketStream(HashSet<String>),
    /// websocket API，不需要额外参数，而是在请求时发送参数
    Api,
}

impl ChannelPath {
    /// 对于行情类订阅，返回：空字符串或者"<StreamName1>/<StreamName2>..."
    /// 对于账户类定于，返回："<ListenKey>"
    fn to_path(&self) -> String {
        match self {
            Self::MarketStream(s) => Vec::from_iter(s.iter().map(|x| x.as_str())).join("/"),
            Self::Api => String::new(),
        }
    }

    /// 空的MarketPath表示暂时不订阅，而是等待以后手动添加订阅
    /// ```
    /// ChannelPath::market_path(HashSet::from(["btcusdt@aggTrade".to_string()]));
    /// ```
    pub fn market_stream_path(content: HashSet<String>) -> Self {
        Self::MarketStream(content)
    }

    /// 生成空的MarketPath，空的MarketPath表示暂时不订阅，而是等待以后手动添加订阅
    pub fn empty_market_stream_path() -> Self {
        Self::MarketStream(HashSet::default())
    }

    /// 向MarketStreamPath中添加数据，如果是ListenKey类型，则不做任何事情
    /// 用于手动订阅时添加指定要订阅的channel
    fn extend(&mut self, contents: HashSet<String>) {
        match self {
            ChannelPath::MarketStream(h) => h.extend(contents),
            ChannelPath::Api => {}
        }
    }

    /// 从MarketStreamPath中移除数据，如果是ListenKey类型，则不做任何事情
    /// 用于手动取消订阅时移除指定要取消订阅的channel
    fn remove(&mut self, data: &str) {
        match self {
            ChannelPath::MarketStream(h) => {
                h.remove(data);
            }
            ChannelPath::Api => {}
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
    fn ws_url(channel_path: &ChannelPath) -> String {
        match channel_path {
            ChannelPath::MarketStream(hash_set) => {
                let path = Vec::from_iter(hash_set.iter().map(|x| x.as_str())).join("/");
                if !path.is_empty() {
                    concat_string!(WS_STREAM_URL, "/stream?streams=", path)
                    // format!("{}/stream?streams={}", WS_BASE_URL, path)
                } else {
                    concat_string!(WS_STREAM_URL, "/stream", path)
                    // format!("{}/stream", WS_BASE_URL)
                }
            }
            ChannelPath::Api => WS_API_URL.into(),
        }
    }

    async fn channel_path(&self) -> String {
        self.channel_path.read().await.to_path()
    }

    /// 建立ws连接，并处理返回数据  
    /// 每次调用都只能订阅单个频道(channel参数)  
    async fn new(channel_path: ChannelPath) -> BiAnResult<Self> {
        match &channel_path {
            ChannelPath::MarketStream(v) => {
                if v.len() > 1024 {
                    return Err(BiAnApiError::TooManySubscribes(v.len()));
                }
            }
            ChannelPath::Api => {}
        }

        let url = Self::ws_url(&channel_path);

        let (ws_stream, _response) = connect_async(&url).await.map_err(Box::new)?;
        let (ws_writer, ws_reader) = ws_stream.split();

        Ok(WS {
            ws_reader: RwLock::new(ws_reader),
            ws_writer: RwLock::new(ws_writer),
            channel_path: RwLock::new(channel_path),
        })
    }

    /// 创建一个新的WebSocketStream，并将其替换该ws连接内部已有的WebSocketStream  
    async fn replace_inner_stream(&self) {
        let dur = Duration::from_millis(500);
        let s = self.channel_path.read().await;
        let url = Self::ws_url(&s);
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
            reason: Utf8Bytes::from_static("force close"),
        };
        let msg = Message::Close(Some(close_frame));
        self.ws_writer
            .write()
            .await
            .send(msg)
            .await
            .map_err(Box::new)?;
        Ok(())
    }

    /// 列出订阅内容，可用于检查是否订阅成功
    /// 向通道发送信息，查看订阅结果，通道会响应id和result字段
    /// id参数随意，响应中的id字段总是和该给定的id相同以示对应
    async fn list_sub(&self, id: u64) {
        let json_text = serde_json::json!({
          "method": "LIST_SUBSCRIPTIONS",
          "id": id
        });
        let msg = Message::Text(json_text.to_string().into());
        if let Err(e) = self.ws_writer.write().await.send(msg).await {
            error!("channel closed: {}, <{}>", e, self.channel_path().await,);
        }
    }

    /// 手动订阅
    async fn stream_subscribe(&self, channel_path: ChannelPath, id: u64) {
        let contents = match channel_path {
            ChannelPath::MarketStream(s) => s,
            ChannelPath::Api => panic!("subscribe websocket api not allowed"),
        };

        let json_text = serde_json::json!({
            "method": "SUBSCRIBE",
            "params": contents,
            "id": id
        });
        let msg = Message::Text(json_text.to_string().into());
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
    async fn stream_unsubscribe(&self, channel_path: ChannelPath, id: u64) {
        let contents = match channel_path {
            ChannelPath::MarketStream(s) => s,
            ChannelPath::Api => panic!("unsubscribe websocket api not allowed"),
        };
        let json_text = serde_json::json!({
            "method": "UNSUBSCRIBE",
            "params": contents,
            "id": id
        });
        let msg = Message::Text(json_text.to_string().into());
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
#[derive(Clone)]
#[must_use = "`WsClient` must be use"]
pub struct WsClient {
    // 不使用DashMap，因为它在不切换任务的过程中重复get_mut时，不会释放锁，
    // 相邻两次之间必须至少有一个额外的异步任务才可以，比如`tokio::task::yield_now().await`
    // pub ws: Arc<DashMap<(), WS>>,
    ws: Arc<WS>,
    api_sec_key: ApiSecKey,
    close_sender: mpsc::Sender<bool>,
    /// 是否登录了
    logon_flag: Arc<AtomicBool>,
    /// 是否订阅了 user data stream
    uds_subscribed: Arc<AtomicBool>,
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
        data_sender: mpsc::Sender<WsResponse>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        let ws = WS::new(channel_path).await?;
        let (close_sender, close_receiver) = mpsc::channel::<bool>(1);
        let s = Self {
            ws: Arc::new(ws),
            close_sender,
            api_sec_key: ApiSecKey::default(),
            logon_flag: Arc::new(AtomicBool::new(false)),
            uds_subscribed: Arc::new(AtomicBool::new(false)),
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
        self.ws.stream_subscribe(channel_path, id).await;
    }

    /// 取消订阅
    /// ```
    /// let channel_path = ChannelPath::market_path(HashSet::from(["btcusdt@aggTrade".to_string()]));
    /// self.unsubscribe(channel_path).await;
    /// ```
    pub async fn unsubscribe(&self, channel_path: ChannelPath, id: u64) {
        self.ws.stream_unsubscribe(channel_path, id).await;
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
    async fn read_from_channel(&self, data_sender: mpsc::Sender<WsResponse>) {
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
                                if let Some(err_msg) = self.handle_msg(msg, &data_sender).await {
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

            // 重建连接时，如果已经登录或已经订阅账户数据，则也重新登录、订阅
            self.ws.replace_inner_stream().await;
            if self.logon_flag.load(atomic::Ordering::Acquire) {
                info!("session re-logon");
                let _ = self.session_logon().await;
            }
            if self.uds_subscribed.load(atomic::Ordering::Acquire) {
                info!("session re-subscribe user data stream");
                let _ = self.user_data_stream_subscribe().await;
            }
        }
    }

    /// 处理ws接收到的信息，并且在接收到ws关闭信息的时候替换重建ws
    /// 返回Some(close_reason)表示要重建ws，返回None表示一切正常无需重建
    async fn handle_msg(
        &self,
        msg: Message,
        data_sender: &mpsc::Sender<WsResponse>,
    ) -> Option<String> {
        match msg {
            Message::Text(data) => {
                // warn!("websocket recv: {}", data.as_str());
                match serde_json::from_slice::<WsResponse>(data.as_bytes()) {
                    Ok(resp) => {
                        // 如果是登录、订阅账户更新的消息，则保存连接当前是否已经登录、是否已经订阅的状态
                        if let WsResponse::ApiResp(r) = &resp
                            && let WebsocketApiResponse::LogonInfo(i) = &r.result
                        {
                            self.logon_flag
                                .store(i.authorized_since.is_some(), atomic::Ordering::Release);
                            self.uds_subscribed
                                .store(i.user_data_stream, atomic::Ordering::Release);
                        }
                        if data_sender.send(resp).await.is_err() {
                            error!("Data Receiver is closed");
                        }
                    }
                    Err(e) => {
                        // 接收到无法解析的内容不停止，继续从websocket里等待数据
                        error!(
                            "Ws Message Decode to WsResponse Error, error: {e}, msg content: {}",
                            data.as_str()
                        );
                    }
                }
            }
            Message::Ping(d) => {
                debug!("received Ping Frame");
                let msg = Message::Pong(d);
                if let Err(e) = self.ws.ws_writer.write().await.send(msg).await {
                    error!("ws closed: {}, <{}>", e, self.ws.channel_path().await);
                }
            }
            Message::Close(Some(data)) => {
                warn!("recv CloseFrame: {}", data.code);
                return Some(data.reason.to_string());
            }
            _ => (),
        }
        None
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
}

impl WsClient {
    /// 以ws_client的方式订阅"归集交易流"(该操作不会"阻塞"当前异步任务)  
    /// 归集交易 stream 推送交易信息，是对单一订单的集合  
    /// symbols参数忽略大小写  
    pub async fn agg_trade(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<WsResponse>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <sym>@aggTrade
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@aggTrade"))
            .collect::<HashSet<String>>();
        Self::new(ChannelPath::market_stream_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"逐笔交易流"(该操作不会"阻塞"当前异步任务)  
    /// 逐笔交易推送每一笔成交的信息。成交，或者说交易的定义是仅有一个吃单者与一个挂单者相互交易  
    /// symbols参数忽略大小写  
    pub async fn trade(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<WsResponse>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <sym>@trade
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@trade"))
            .collect::<HashSet<String>>();
        Self::new(ChannelPath::market_stream_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"K线数据流"(该操作不会"阻塞"当前异步任务)  
    /// K线stream逐秒推送所请求的K线种类(最新一根K线)的更新  
    /// symbols参数忽略大小写  
    pub async fn kline(
        interval: &str,
        symbols: Vec<String>,
        data_sender: mpsc::Sender<WsResponse>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        if !KLineInterval::is_intv(interval) {
            panic!("argument error: <{}> invalid kline interval", interval);
        }

        // channel: kline_<interval>
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@kline_", interval))
            .collect::<HashSet<String>>();

        Self::new(ChannelPath::market_stream_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"按symbol的精简Ticker"(该操作不会"阻塞"当前异步任务)  
    /// 按Symbol刷新的最近24小时精简ticker信息  
    /// symbols参数忽略大小写  
    pub async fn mini_ticker(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<WsResponse>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <sym>@miniTicker
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@miniTicker"))
            .collect::<HashSet<String>>();
        Self::new(ChannelPath::market_stream_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"全市场所有Symbol的精简Ticker"(该操作不会"阻塞"当前异步任务)  
    /// 推送所有交易对的最近24小时精简ticker信息.需注意，只有更新的ticker才会被推送
    pub async fn all_mini_ticker(
        data_sender: mpsc::Sender<WsResponse>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: !miniTicker@arr
        let channel_path = HashSet::from(["!miniTicker@arr".into()]);
        Self::new(ChannelPath::market_stream_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"按symbol的完整Ticker"(该操作不会"阻塞"当前异步任务)  
    /// 每秒推送单个交易对的过去24小时滚动窗口标签统计信息  
    /// symbols参数忽略大小写  
    pub async fn ticker(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<WsResponse>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <sym>@ticker
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@ticker"))
            .collect::<HashSet<String>>();
        Self::new(ChannelPath::market_stream_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"全市场所有Symbol的完整Ticker"(该操作不会"阻塞"当前异步任务)  
    /// 推送所有交易对的最近24小时完整ticker信息.需注意，只有更新的ticker才会被推送
    pub async fn all_ticker(
        data_sender: mpsc::Sender<WsResponse>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: !ticker@arr
        let channel_path = HashSet::from(["!ticker@arr".into()]);
        Self::new(ChannelPath::market_stream_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"按Symbol的最优挂单信息"(该操作不会"阻塞"当前异步任务)  
    /// 实时推送指定交易对最优挂单信息  
    /// symbols参数忽略大小写  
    pub async fn bookticker(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<WsResponse>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <sym>@bookTicker
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@bookTicker"))
            .collect::<HashSet<String>>();
        Self::new(ChannelPath::market_stream_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"有限档深度信息"(该操作不会"阻塞"当前异步任务)  
    /// 每100毫秒推送有限档深度信息。level表示几档买卖单信息, 可选5/10/20档，
    /// 档数表示返回结果中包含几个挂单信息，5档表示返回最新的5个买盘和5个卖盘挂单信息，  
    /// symbols参数忽略大小写  
    pub async fn depth_with_level(
        symbols: Vec<String>,
        level: u8,
        data_sender: mpsc::Sender<WsResponse>,
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

        Self::new(ChannelPath::market_stream_path(channel_path), data_sender).await
    }

    /// 以ws_client的方式订阅"增量深度信息"(该操作不会"阻塞"当前异步任务)  
    /// 每100毫秒推送orderbook的变化部分(如果有)  
    /// symbols参数忽略大小写  
    pub async fn depth_incr(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<WsResponse>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        // channel: <symbol>@depth 或 <symbol>@depth@100ms
        let channel_path = symbols
            .iter()
            .map(|sym| concat_string!(sym.to_ascii_lowercase(), "@@depth@100ms"))
            .collect::<HashSet<String>>();

        Self::new(ChannelPath::market_stream_path(channel_path), data_sender).await
    }
}

impl WsClient {
    /// 订阅websocket api连接，
    /// 1.可以发送类似于rest api的请求(比如下单、撤单、获取K线等), 并获得响应
    /// 2.也可以登录之后订阅账户信息变更时推送的数据流
    ///
    /// 所有这类请求发送后都会返回一个Uuid，api websocket每次对其响应都会带上Uuid以便区分，
    /// 但 api websocket 除了对每次请求的响应，如果用户还请求过订阅账户信息更新后推送的数据，
    /// 那么所有这类账户更新的推送内容不会带上Uuid，而是以Event的方式作为响应
    ///
    /// ```rust
    /// let (data_sender, mut data_receiver) = mpsc::channel::<WsResponse>(1000);
    /// let (ws, task_handle) = WsClient::ws_api(api_sec_key, data_sender).await;
    ///
    /// // 订阅账户数据更新、余额更新、订单更新等推送
    /// tokio::spawn(async move {
    ///     ws.session_logon().await;
    ///     ws.user_data_stream_subscribe().await;
    /// });
    /// // 获取一部分K线
    /// tokio::spawn(async move {
    ///     let _uuid = ws.klines("BTCUSDT", "1m", Some(1735896120000), None, Some(10)).await;
    /// });
    /// while let Some(x) = data_receiver.recv().await {
    ///     info!("account received: {:?}", x);
    ///     match x {
    ///         // 市场数据更新推送
    ///         WsResponse::MarketStream(ws_market_stream_resp) => todo!(),
    ///         // 账户数据更新推送(比如余额更新，订单更新)
    ///         WsResponse::UserDataStream(ws_user_data_stream_resp) => todo!(),
    ///         // 订阅市场数据、取消订阅市场数据、查询订阅状态时的单次响应
    ///         WsResponse::MarketStreamSubscribe(ws_subscribe_resp) => todo!(),
    ///         // 发送类似于Rest Api功能的请求后的单次响应，比如获取一段区间的K线数据
    ///         WsResponse::ApiResp(ws_api_resp) => todo!(),
    ///     }
    /// }
    /// ```
    pub async fn ws_api(
        api_sec_key: ApiSecKey,
        data_sender: mpsc::Sender<WsResponse>,
    ) -> BiAnResult<(Self, JoinHandle<()>)> {
        let (mut s, j) = Self::new(ChannelPath::Api, data_sender).await?;
        s.api_sec_key = api_sec_key;
        Ok((s, j))
    }

    /// 向websocket api连接发送请求，必须已经设置好了api_key(即必须通过`ws_api()`方法创建websocket连接)，否则将返回Error
    pub async fn send_api_req<T>(
        &self,
        method: &'static str,
        params: Option<&T>,
    ) -> BiAnResult<Uuid>
    where
        T: Serialize + Param,
    {
        if self.api_sec_key.is_api_empty() {
            return Err(BiAnApiError::ApiKeyError);
        }
        let param = PWebSocketApi::new(&self.api_sec_key, method, params);
        let msg_str = serde_json::to_string(&param).unwrap();
        let msg = Message::Text(msg_str.as_str().into());
        match self.ws.ws_writer.write().await.send(msg).await {
            Ok(_) => {
                info!("msg send to websocket: {msg_str}")
            }
            Err(e) => {
                error!("websocket connect closed: {}", e);
            }
        }

        Ok(param.id)
    }

    /// 会话登录，将等待登录成功
    pub async fn session_logon(&self) -> BiAnResult<Uuid> {
        let uuid = self
            .send_api_req("session.logon", Some(&PSessionLogon::new()))
            .await?;

        let short_dur = tokio::time::Duration::from_millis(100);
        let long_dur = tokio::time::Duration::from_secs(10);
        let mut n = 0;
        while !self.logon_flag.load(atomic::Ordering::Acquire) {
            if n >= 100 {
                tokio::time::sleep(long_dur).await;
            } else {
                tokio::time::sleep(short_dur).await;
            }
            n += 1;
        }
        info!("session logon successed: {uuid}");
        Ok(uuid)
    }

    /// 会话状态查看
    pub async fn session_status(&self) -> BiAnResult<Uuid> {
        self.send_api_req("session.status", <Option<&PSessionStatus>>::None)
            .await
    }

    /// 订阅账户数据流，要求先通过`session.logon`登录，
    /// 并且订阅操作和登录操作中间应当有一小段延迟，否则可能还未登录成功就请求订阅，会失败，
    ///
    /// 将等待订阅成功，订阅成功后不会阻塞
    pub async fn user_data_stream_subscribe(&self) -> BiAnResult<Uuid> {
        let uuid = self
            .send_api_req("userDataStream.subscribe", <Option<&PUserDataStream>>::None)
            .await?;

        // 为了更新订阅是否成功的状态，不断发送`session.status`进行查询
        let short_dur = tokio::time::Duration::from_millis(100);
        let long_dur = tokio::time::Duration::from_secs(10);
        let mut n = 0;
        while !self.uds_subscribed.load(atomic::Ordering::Acquire) {
            if n >= 100 {
                tokio::time::sleep(long_dur).await;
            } else {
                tokio::time::sleep(short_dur).await;
            }
            // 如果睡完之后已经变成了true，则不要再发送多余的查询请求
            if self.uds_subscribed.load(atomic::Ordering::Acquire) {
                break;
            }
            let _ = self.session_status().await;
            n += 1;
        }

        info!("user data stream subscribe successed: {uuid}");
        Ok(uuid)
    }

    /// 取消订阅账户数据流
    pub async fn user_data_stream_unsubscribe(&self) -> BiAnResult<Uuid> {
        let uuid = self
            .send_api_req(
                "userDataStream.unsubscribe",
                <Option<&PUserDataStream>>::None,
            )
            .await?;
        self.uds_subscribed.store(false, atomic::Ordering::Release);
        Ok(uuid)
    }

    /// 要求先设置 api key，即必须先通过`ws_api()`创建websocket连接，再发起请求
    pub async fn klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> BiAnResult<Uuid> {
        let pkline = PKLine::new(symbol, interval, start_time, end_time, limit)?;
        self.send_api_req("klines", Some(&pkline)).await
    }
}
