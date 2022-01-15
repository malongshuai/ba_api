use core::panic;

use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
    MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, warn};

use crate::{
    errors::{BiAnApiError, BiAnResult},
    KLineInterval, WS_BASE_URL,
};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// 内部ws连接，只支持订阅组合Stream(参考<https://binance-docs.github.io/apidocs/spot/cn/#websocket>)
#[derive(Debug)]
pub struct WS {
    /// WebSocketStream
    pub conn_stream: WsStream,
    pub names: Vec<String>,
    pub channel: String,
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

        let base_url = *WS_BASE_URL;
        let names: Vec<String> = names.iter().map(|x| x.to_string()).collect();
        let url = Self::make_ws_url(channel, &names, base_url);

        let (ws_stream, _response) = connect_async(&url).await?;

        Ok(WS {
            conn_stream: ws_stream,
            names,
            channel: channel.to_string(),
            url,
        })
    }

    /// 创建一个新的WebSocketStream，并将其替换该ws连接内部已有的WebSocketStream  
    pub async fn replace_inner_stream(&mut self) -> BiAnResult<()> {
        let (ws_stream, _response) = connect_async(&self.url).await?;
        self.conn_stream = ws_stream;
        Ok(())
    }

    /// 关闭ws conn_stream
    async fn close_stream(&mut self) -> BiAnResult<()> {
        let close_frame = CloseFrame {
            code: CloseCode::Normal,
            reason: std::borrow::Cow::Borrowed("force close"),
        };
        self.conn_stream.close(Some(close_frame)).await?;
        Ok(())
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
#[derive(Debug)]
pub struct WsClient {
    pub ws: WS,
    rx: mpsc::Receiver<bool>,
    /// 该ws连接订阅的频道名称
    pub channel: String,
    /// 该ws订阅的频道
    pub names: Vec<String>,
    /// 该ws的URL
    pub url: String,
}

impl WsClient {
    /// 新建到币安的ws连接客户端，需传入一个mpsc::Receiver来接收连接关闭的通知
    pub async fn new(
        channel: &str,
        names: Vec<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<Self> {
        let ws = WS::new(channel, names).await?;
        let names = ws.names.clone();
        let url = ws.url.clone();

        Ok(Self {
            ws,
            rx: close_receiver,
            channel: channel.to_string(),
            names,
            url,
        })
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

    async fn handle_msg(
        &mut self,
        msg: Message,
        data_sender: mpsc::Sender<String>,
    ) -> BiAnResult<()> {
        match msg {
            Message::Text(data) => {
                // debug!("received Text Frame: {}", data);
                if data_sender.send(data).await.is_err() {
                    debug!("Data Receiver already closed");
                }
            }
            Message::Ping(_) => {
                debug!("received Ping Frame");
                let pong = Message::Pong(vec![]);
                self.ws.conn_stream.send(pong).await.unwrap();
            }
            Message::Close(Some(data)) => {
                debug!(
                    "close reason: <{}>, <{}>",
                    data.reason,
                    self.names.join(",")
                );
                self.ws.replace_inner_stream().await?;
            }
            _ => (),
        }
        Ok(())
    }

    /// 建立一个断连后会自动重连的ws客户端(会阻塞当前异步任务)  
    /// 该ws客户端接收到的数据(String)，都将通过mpsc的sender发送出去  
    /// 如果需要强制关闭ws_client，使用close_client发送true参数，如果需要手动断开但自动重连(相当于重建连接)，使用close_client发送false参数  
    /// ```rust
    /// let (data_tx, mut data_rx) = mpsc::channel::<String>(1000);
    ///
    /// let (close_sender, close_receiver) = mpsc::channel::<bool>(1);
    /// let mut wsc = WsClient::new("kline_1m", vec!["btcusdt", "ethusdt"], close_receiver).await.unwrap();
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
    pub async fn ws_client(&mut self, data_sender: mpsc::Sender<String>) -> BiAnResult<()> {
        loop {
            if data_sender.is_closed() {
                break;
            }
            tokio::select! {
                Some(msg) = self.ws.conn_stream.next() => {
                    match msg {
                        Ok(msg) => self.handle_msg(msg, data_sender.clone()).await?,
                        Err(_) => warn!("ws closed: {}", self.names.join(","))
                    }
                },
                Some(data) = self.rx.recv() => {
                    self.ws.close_stream().await?;
                    if data { break; }
                }
            }
        }
        Ok(())
    }

    /// 创建一个ws连接并进行订阅，将"阻塞"当前异步任务，连接断开将自动重连，如需关闭连接ws，通过close_client并使用close_receiver配对的Sender作为参数发送true来强制关闭连接
    /// ```rust
    /// // 收发数据的通道
    /// let (data_tx, mut data_rx) = mpsc::channel::<String>(1000);
    /// // 收发关闭连接通知的通道
    /// let (close_tx, close_rx) = mpsc::channel::<bool>(1);
    /// WsClient::sub_channel("kline_1m", vec!["btcusdt", "ethusdt"], data_tx, close_rx).await.unwrap();
    /// ```
    pub async fn sub_channel(
        channel: &str,
        names: Vec<String>,
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        let mut wsc = Self::new(channel, names, close_receiver).await?;
        wsc.ws_client(data_sender).await?;
        Ok(())
    }

    /// 以ws_client的方式订阅"归集交易流"(将"阻塞"当前异步任务)  
    /// 归集交易 stream 推送交易信息，是对单一订单的集合  
    /// symbols参数忽略大小写  
    /// ```rust
    /// // 收发数据的通道
    /// let (data_tx, mut data_rx) = mpsc::channel::<String>(1000);
    /// // 收发关闭连接通知的通道
    /// let (close_sender, close_receiver) = mpsc::channel::<bool>(1);
    ///
    /// WsClient::agg_trade(vec!["btcusdt", "ethusdt"], data_tx, close_receiver);
    /// ```
    pub async fn agg_trade(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Ok(Self::sub_channel("aggTrade", symbols, data_sender, close_receiver).await?)
    }

    /// 以ws_client的方式订阅"逐笔交易流"(将"阻塞"当前异步任务)  
    /// 逐笔交易推送每一笔成交的信息。成交，或者说交易的定义是仅有一个吃单者与一个挂单者相互交易  
    /// symbols参数忽略大小写  
    pub async fn trade(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Ok(Self::sub_channel("trade", symbols, data_sender, close_receiver).await?)
    }

    /// 以ws_client的方式订阅"K线数据流"(将"阻塞"当前异步任务)  
    /// K线stream逐秒推送所请求的K线种类(最新一根K线)的更新  
    /// symbols参数忽略大小写  
    /// ```rust
    /// let (data_sender, mut data_receiver) = mpsc::channel::<String>(1000);
    /// let (close_sender, close_receiver) = mpsc::channel::<bool>(1);
    ///
    /// tokio::spawn(async move {
    ///     while let Some(x) = data_receiver.recv().await {
    ///         println!("channel received: {}", x);
    ///     }
    /// });
    ///
    /// // 20秒后强制关闭ws连接
    /// tokio::spawn(async move {
    ///     tokio::time::sleep(Duration::from_secs(20)).await;
    ///     WsClient::close_client(close_sender, true).await;
    ///     debug!("send close");
    /// });
    ///
    /// WsClient::kline(
    ///     "1m",
    ///     vec!["btcusdt", "ethusdt"],
    ///     data_sender,
    ///     close_receiver,
    /// )
    /// .await
    /// .unwrap();
    /// ```
    pub async fn kline(
        interval: &str,
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        if !KLineInterval::is_intv(interval) {
            panic!(
                "argument error: <{}> is not the valid kline interval",
                interval
            );
        }
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        let channel = format!("kline_{}", interval);
        Ok(Self::sub_channel(&channel, symbols, data_sender, close_receiver).await?)
    }

    /// 以ws_client的方式订阅"按symbol的精简Ticker"(将"阻塞"当前异步任务)  
    /// 按Symbol刷新的最近24小时精简ticker信息  
    /// symbols参数忽略大小写  
    pub async fn mini_ticker(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Ok(Self::sub_channel("miniTicker", symbols, data_sender, close_receiver).await?)
    }

    /// 以ws_client的方式订阅"全市场所有Symbol的精简Ticker"(将"阻塞"当前异步任务)  
    /// 推送所有交易对的最近24小时精简ticker信息.需注意，只有更新的ticker才会被推送
    pub async fn all_mini_ticker(
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        Ok(Self::sub_channel(
            "arr",
            vec!["!miniTicker".to_string()],
            data_sender,
            close_receiver,
        )
        .await?)
    }

    /// 以ws_client的方式订阅"按symbol的完整Ticker"(将"阻塞"当前异步任务)  
    /// 每秒推送单个交易对的过去24小时滚动窗口标签统计信息  
    /// symbols参数忽略大小写  
    pub async fn ticker(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Ok(Self::sub_channel("miniTicker", symbols, data_sender, close_receiver).await?)
    }

    /// 以ws_client的方式订阅"全市场所有Symbol的完整Ticker"(将"阻塞"当前异步任务)  
    /// 推送所有交易对的最近24小时完整ticker信息.需注意，只有更新的ticker才会被推送
    pub async fn all_ticker(
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        Ok(Self::sub_channel(
            "arr",
            vec!["!ticker".to_string()],
            data_sender,
            close_receiver,
        )
        .await?)
    }

    /// 以ws_client的方式订阅"按Symbol的最优挂单信息"(将"阻塞"当前异步任务)  
    /// 实时推送指定交易对最优挂单信息  
    /// symbols参数忽略大小写  
    pub async fn bookticker(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Ok(Self::sub_channel("bookTicker", symbols, data_sender, close_receiver).await?)
    }

    /// 以ws_client的方式订阅"全市场最优挂单信息"(将"阻塞"当前异步任务)  
    /// 实时推送所有交易对最优挂单信息
    pub async fn all_bookticker(
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        let names: Vec<String> = vec![];
        Ok(Self::sub_channel("!bookTicker", names, data_sender, close_receiver).await?)
    }

    /// 以ws_client的方式订阅"有限档深度信息"(将"阻塞"当前异步任务)  
    /// 每100毫秒推送有限档深度信息。level表示几档买卖单信息, 可选5/10/20档  
    /// symbols参数忽略大小写  
    pub async fn depth_with_level(
        symbols: Vec<String>,
        level: u8,
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        if ![5u8, 10u8, 20u8].contains(&level) {
            panic!("argument error: <{}> is not the valid depth level", level)
        }
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        let channel = format!("depth{}@100ms", level);
        Ok(Self::sub_channel(&channel, symbols, data_sender, close_receiver).await?)
    }

    /// 以ws_client的方式订阅"增量深度信息"(将"阻塞"当前异步任务)  
    /// 每100毫秒推送orderbook的变化部分(如果有)  
    /// symbols参数忽略大小写  
    pub async fn depth_incr(
        symbols: Vec<String>,
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        let symbols: Vec<String> = symbols.iter().map(|x| x.to_lowercase()).collect();
        Ok(Self::sub_channel("depth@100ms", symbols, data_sender, close_receiver).await?)
    }

    /// 以ws_client的方式订阅"Websocket账户信息推送"(将"阻塞"当前异步任务)  
    ///
    /// 包括账户更新、余额更新、订单更新，参考(<https://binance-docs.github.io/apidocs/spot/cn/#websocket-2>)  
    pub async fn account(
        listen_key: String,
        data_sender: mpsc::Sender<String>,
        close_receiver: mpsc::Receiver<bool>,
    ) -> BiAnResult<()> {
        Ok(Self::sub_channel(&listen_key, vec![], data_sender, close_receiver).await?)
    }
}
