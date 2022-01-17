use std::fmt::Display;

use crate::client::option_string_to_f64;
use crate::client::string_to_f64;
use serde::{Deserialize, Serialize};

/// 订单状态
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    /// 订单请求被接受，即挂单成功
    New,

    /// 订单部分成交
    PartiallyFilled,

    /// 订单完全成交
    Filled,

    /// 订单被撤单
    Canceled,

    /// 订单正在撤单(当前币安未使用该状态)
    PendingCancel,

    /// 订单请求被拒绝
    Rejected,

    /// 订单被交易引擎取消，例如limit fok订单没有成交，市价单未完全成交，强平期被取消的订单，交易所系统维护被取消的订单
    Expired,
}

/// 订单被触发了什么操作
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderAction {
    /// 创建新订单的操作
    New,
    /// 订单有成交信息的操作
    Trade,
    /// 订单被撤销的操作
    Canceled,
    /// 订单请求被拒绝的操作
    Rejected,
    /// 订单失效的操作
    Expired,
    /// 尚未使用的保留字段
    Reeplaced,
}

/// 订单类型(参考<https://www.binance.com/cn/support/articles/360033779452-Types-of-Order>)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    /// 限价单
    Limit,
    /// 市价单
    Market,
    /// 止损单
    StopLoss,
    /// 限价止损单
    StopLossLimit,
    /// 止盈单
    TakeProfit,
    ///限价止盈单
    TakeProfitLimit,
    /// 限价只挂单
    LimitMaker,
}
impl From<&str> for OrderType {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "LIMIT" => Self::Limit,
            "MARKET" => Self::Market,
            "STOPLOSS" => Self::StopLoss,
            "STOPLOSSLIMIT" => Self::StopLossLimit,
            "TAKEPROFIT" => Self::TakeProfit,
            "TAKEPROFITLIMIT" => Self::TakeProfitLimit,
            "LIMITMAKER" => Self::LimitMaker,
            s => panic!("`{}' unsupported Order Type", s),
        }
    }
}

/// 订单信息的返回类型(下单时的返回信息详细程度)  
/// 默认情况下，市价单(Market)和限价单(Limit)为Full，其它类型的订单(如止损单、止盈单)为Ack
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderRespType {
    Ack,
    Result,
    Full,
}
impl From<&str> for OrderRespType {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ACK" => Self::Ack,
            "RESULT" => Self::Result,
            "FULL" => Self::Full,
            s => panic!("`{}' unsupported Order Resp Type", s),
        }
    }
}

/// 订单方向
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl From<&str> for OrderSide {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "BUY" => Self::Buy,
            "SELL" => Self::Sell,
            s => panic!("`{}' unsupported Order Side", s),
        }
    }
}
impl Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OrderSide::Buy => "BUY".to_string(),
            OrderSide::Sell => "SELL".to_string(),
        };
        write!(f, "{}", s)
    }
}

/// 订单有效方式(订单何时失效)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TimeInForce {
    /// 订单一直有效，直到订单完全成交或被撤单
    GTC,

    /// 无法立即成交的部分会被立即撤单，然后订单失效
    IOC,

    /// 无法全部立即成交就直接撤单，然后订单失效
    FOK,
}

impl From<&str> for TimeInForce {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GTC" => Self::GTC,
            "IOC" => Self::IOC,
            "FOK" => Self::FOK,
            s => panic!("`{}' unsupported TimeInForce", s),
        }
    }
}

/// 逐笔交易
#[derive(Debug, Deserialize, Serialize)]
#[serde(from = "WrapTrade")]
pub struct Trade {
    /// 交易对
    pub symbol: String,
    /// 交易ID
    pub id: u64,
    /// 成交价格
    pub price: f64,
    /// 成交数量
    pub qty: f64,
    /// 成交时间
    pub time: u64,
    /// 买方是否是做市方
    pub is_buyer_maker: bool,
    /// 是否是最优匹配，忽略该字段
    pub is_best_match: bool,
}

impl From<WrapTrade> for Trade {
    fn from(trade: WrapTrade) -> Self {
        match trade {
            WrapTrade::RestTrade(data) => Self {
                symbol: String::new(),
                id: data.id,
                price: data.price,
                qty: data.qty,
                time: data.time,
                is_buyer_maker: data.is_buyer_maker,
                is_best_match: data.is_best_match,
            },
            WrapTrade::WebSocketTrade(data) => Self {
                symbol: data.symbol,
                id: data.id,
                price: data.price,
                qty: data.qty,
                time: data.time,
                is_buyer_maker: data.is_buyer_maker,
                is_best_match: data.is_best_match,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum WrapTrade {
    RestTrade(RestTrade),
    WebSocketTrade(WebSocketTrade),
}

/// 逐笔交易(Rest接口)
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RestTrade {
    /// 交易ID
    id: u64,
    /// 成交价格
    #[serde(deserialize_with = "string_to_f64")]
    price: f64,
    /// 成交数量
    #[serde(deserialize_with = "string_to_f64")]
    qty: f64,
    /// 成交时间
    time: u64,
    /// 买方是否是做市方
    is_buyer_maker: bool,
    /// 是否是最优匹配，忽略该字段
    is_best_match: bool,
}

/// 逐笔交易(WebSocket接口)
#[derive(Debug, Serialize, Deserialize)]
struct WebSocketTrade {
    /// 交易对
    #[serde(rename = "s")]
    symbol: String,
    /// 交易ID
    #[serde(rename = "t")]
    id: u64,
    /// 成交价格
    #[serde(rename = "p", deserialize_with = "string_to_f64")]
    price: f64,
    /// 成交数量
    #[serde(rename = "q", deserialize_with = "string_to_f64")]
    qty: f64,
    /// 成交时间
    #[serde(rename = "T")]
    time: u64,
    /// 买方是否是做市方
    #[serde(rename = "m")]
    is_buyer_maker: bool,
    /// 是否是最优匹配，忽略该字段
    #[serde(rename = "M")]
    is_best_match: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalTrade {
    pub id: u64,
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub qty: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub quote_qty: f64,
    pub time: u64,
    pub is_buyer_maker: bool,
    pub is_best_match: bool,
}

/// 归集订单数据
#[derive(Debug, Deserialize, Serialize)]
pub struct AggTrade {
    /// 交易对
    #[serde(default)]
    #[serde(rename = "s")]
    pub symbol: String,

    /// 归集成交ID
    #[serde(rename = "a")]
    pub id: u64,

    /// 成交价
    #[serde(rename = "p")]
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,

    /// 成交量
    #[serde(rename = "q")]
    #[serde(deserialize_with = "string_to_f64")]
    pub qty: f64,

    /// 该归集的首个成交ID
    #[serde(rename = "f")]
    pub first_id: u64,

    /// 该归集的末个成交ID
    #[serde(rename = "l")]
    pub last_id: u64,

    /// 该归集的成交时间
    #[serde(rename = "T")]
    pub time: u64,

    /// 是否为主动卖出单
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,

    /// 是否为最优撮合单(可忽略，目前总是为最优撮合)
    #[serde(rename = "M")]
    pub is_best_match: bool,
}

/// 订单的交易信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeInfo {
    /// 交易的价格
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,

    /// 交易的数量
    #[serde(deserialize_with = "string_to_f64")]
    pub qty: f64,

    /// 手续费金额
    #[serde(deserialize_with = "string_to_f64")]
    pub commission: f64,

    /// 手续费的币种
    pub commission_asset: String,
}

/// ACK方式的订单响应
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RespAck {
    pub symbol: String,
    pub order_id: u64,
    pub order_list_id: i64, // OCO订单ID，否则为 -1
    pub client_order_id: String,
    pub transact_time: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RespResult {
    pub symbol: String,
    pub order_id: u64,
    pub order_list_id: i64,
    pub client_order_id: String,
    pub transact_time: u64,
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub orig_qty: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub executed_qty: f64,
    pub cummulative_quote_qty: String,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    pub r#type: OrderType,
    pub side: OrderSide,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RespFull {
    #[serde(flatten)]
    pub result: RespResult,
    pub fills: Vec<TradeInfo>,
}

/// 普通订单挂单后的返回信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Order {
    /// 因为包含关系，因此顺序不能反：先是Full，再是Result，再是Ack
    RespFull(RespFull),
    RespResult(RespResult),
    RespAck(RespAck),
}

impl Order {
    /// 从返回的订单信息中获取Order ID
    pub fn order_id(&self) -> u64 {
        match self {
            Order::RespFull(d) => d.result.order_id,
            Order::RespResult(d) => d.order_id,
            Order::RespAck(d) => d.order_id,
        }
    }

    /// 从返回的订单信息中获取Symbol
    pub fn symbol(&self) -> String {
        match self {
            Order::RespFull(d) => d.result.symbol.clone(),
            Order::RespResult(d) => d.symbol.clone(),
            Order::RespAck(d) => d.symbol.clone(),
        }
    }
}

/// 普通订单取消的信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderInfo {
    pub symbol: String,
    pub orig_client_order_id: String,
    pub order_id: u64,
    pub order_list_id: i64, // OCO订单ID，否则为 -1
    pub client_order_id: String,
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub orig_qty: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub executed_qty: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub cummulative_quote_qty: f64,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub side: OrderSide,

    #[serde(default)]
    #[serde(deserialize_with = "option_string_to_f64")]
    pub stop_price: Option<f64>,

    #[serde(default)]
    #[serde(deserialize_with = "option_string_to_f64")]
    pub iceberg_qty: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderIdInfo {
    pub symbol: String,
    pub order_id: u64,
    pub client_order_id: String,
}

/// OCO订单取消的信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOCOOrderInfo {
    pub order_list_id: i64,
    pub contingency_type: String,  // OCO
    pub list_status_type: String,  // ALL_DONE
    pub list_order_status: String, // ALL_DONE
    pub list_client_order_id: String,
    pub transaction_time: u64,
    pub symbol: String,
    pub orders: Vec<OrderIdInfo>,
    pub order_reports: Vec<CancelOrderInfo>,
}

/// 订单撤单后的信息，包含：普通订单取消的信息和OCO订单取消的信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CancelOpenOrdersInfo {
    CancelOCOOrderInfo(CancelOCOOrderInfo),
    CancelOrderInfo(CancelOrderInfo),
}

/// 订单信息(来自Rest API接口返回)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderInfo {
    pub symbol: String,
    pub order_id: u64,
    pub client_order_id: String,
    /// OCO订单的ID，不然就是-1
    pub order_list_id: i64,
    /// 订单状态
    pub status: OrderStatus, // "NEW"
    /// 订单的时效方式
    pub time_in_force: TimeInForce,
    /// 订单类型， 比如市价单，现价单等
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// 订单方向，买还是卖
    pub side: OrderSide,
    /// 订单时间
    pub time: u64,
    /// 最后更新时间
    pub update_time: u64,
    /// 订单价格
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,
    /// 用户设置的原始订单数量
    #[serde(deserialize_with = "string_to_f64")]
    pub orig_qty: f64,
    /// 已交易的数量
    #[serde(deserialize_with = "string_to_f64")]
    pub executed_qty: f64,
    /// 累计交易的金额
    #[serde(deserialize_with = "string_to_f64")]
    pub cummulative_quote_qty: f64,
    /// 止盈、止损价格
    #[serde(deserialize_with = "string_to_f64")]
    pub stop_price: f64,
    /// 冰山数量
    #[serde(deserialize_with = "string_to_f64")]
    pub iceberg_qty: f64,
    /// 原始的交易金额
    #[serde(deserialize_with = "string_to_f64")]
    pub orig_quote_order_qty: f64,
    /// 订单是否出现在orderbook中
    pub is_working: bool,
}

/// 账户成交历史
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MyTrades {
    /// 交易对
    pub symbol: String,
    /// trade ID
    pub id: u64,
    /// 订单ID
    pub order_id: u64,
    /// OCO订单的ID，不然就是-1
    pub order_list_id: i64,
    /// 成交价格
    pub price: f64,
    /// 成交量
    pub qty: f64,
    /// 成交金额
    pub quote_qty: f64,
    /// 交易费金额
    pub commission: f64,
    /// 交易费资产类型
    pub commission_asset: String,
    /// 交易时间
    pub time: u64,
    /// 是否是买家
    pub is_buyer: bool,
    /// 是否是挂单方
    pub is_maker: bool,
    /// 是否是最优挂单
    pub is_best_match: bool,
}

/// 订单更新信息(来自WebSocket的推送)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderUpdate {
    #[serde(rename(deserialize = "s"))]
    pub symbol: String,
    #[serde(rename(deserialize = "c"))]
    pub client_order_id: String,
    #[serde(rename(deserialize = "S"))]
    pub side: OrderSide,
    #[serde(rename(deserialize = "o"))]
    pub order_type: OrderType,
    #[serde(rename(deserialize = "f"))]
    pub time_in_force: TimeInForce,
    /// 订单原始数量
    #[serde(rename(deserialize = "q"), deserialize_with = "string_to_f64")]
    pub qty: f64,
    /// 订单原始价格
    #[serde(rename(deserialize = "p"), deserialize_with = "string_to_f64")]
    pub price: f64,
    /// 订单止盈、止损价格
    #[serde(rename(deserialize = "P"), deserialize_with = "string_to_f64")]
    pub stop_price: f64,
    /// 冰山单数量
    #[serde(rename(deserialize = "F"), deserialize_with = "string_to_f64")]
    pub iceberg_qty: f64,
    /// OCO订单ID
    #[serde(rename(deserialize = "g"))]
    pub order_list_id: i64,
    /// 原始订单的client order id，撤单操作有自己的cid
    #[serde(rename(deserialize = "C"))]
    pub orig_client_order_id: String,
    /// 触发推送该订单信息的操作
    #[serde(rename(deserialize = "x"))]
    pub order_action: OrderAction,
    /// 订单的状态
    #[serde(rename(deserialize = "X"))]
    pub order_status: OrderStatus,
    /// 订单被拒绝的原因
    #[serde(rename(deserialize = "r"))]
    pub reason: String,
    /// 订单ID
    #[serde(rename(deserialize = "i"))]
    pub order_id: u64,
    /// 订单末次成交量
    #[serde(rename(deserialize = "l"), deserialize_with = "string_to_f64")]
    pub last_qty: f64,
    /// 订单已累计的成交量
    #[serde(rename(deserialize = "z"), deserialize_with = "string_to_f64")]
    pub cummulative_qty: f64,
    /// 订单末次成交价
    #[serde(rename(deserialize = "L"), deserialize_with = "string_to_f64")]
    pub last_price: f64,
    /// 手续费数量
    #[serde(rename(deserialize = "n"), deserialize_with = "string_to_f64")]
    pub fee_qty: f64,
    /// 手续费资产名称，不产生手续费的订单状态(例如挂单和完全未成交的撤单)其值为null，可能为字符串
    #[serde(rename(deserialize = "N"))]
    pub fee_quote: Option<String>,
    /// 该成交的成交时间
    #[serde(rename(deserialize = "T"))]
    pub trade_time: u64,
    /// 该成交的成交ID(trade ID)
    #[serde(rename(deserialize = "t"))]
    pub trade_id: i64,
    /// 订单是否在订单薄上
    #[serde(rename(deserialize = "w"))]
    pub in_order_book: bool,
    /// 该成交是否是作为挂单成交(是否是maker方)
    #[serde(rename(deserialize = "m"))]
    pub maker: bool,
    /// 订单创建时间
    #[serde(rename(deserialize = "O"))]
    pub order_create_time: u64,
    /// 订单累计成交额
    #[serde(rename(deserialize = "Z"), deserialize_with = "string_to_f64")]
    pub cummulative_vol: f64,
    /// 订单末次成交额
    #[serde(rename(deserialize = "Y"), deserialize_with = "string_to_f64")]
    pub last_vol: f64,
    #[serde(rename(deserialize = "Q"), deserialize_with = "string_to_f64")]
    /// 市价单时，报价资产的数量(例如市价买入BTCUSDT共100USDT时，该字段值为100.0)
    pub quote_order_qty: f64,
}

impl OrderUpdate {
    pub fn is_finished(&self) -> bool {
        matches!(
            self.order_status,
            OrderStatus::Canceled | OrderStatus::Filled
        )
    }
}

#[cfg(test)]
mod test {
    use crate::types::order::Trade;

    #[test]
    fn test_rest_trade() {
        let rest_trade = r##"
            {
              "id": 28457,
              "price": "4.00000100",
              "qty": "12.00000000",
              "time": 1499865549590,
              "isBuyerMaker": true,
              "isBestMatch": true
            }
        "##;
        let x = serde_json::from_str::<Trade>(rest_trade);
        // println!("{:?}", x);
        assert!(x.is_ok());
    }

    #[test]
    fn test_ws_trade() {
        let websocket_trade = r##"
            {
              "e": "trade",  
              "E": 123456789,
              "s": "BNBBTC", 
              "t": 12345,    
              "p": "0.001",  
              "q": "100",    
              "b": 88,       
              "a": 50,       
              "T": 123456785,
              "m": true,     
              "M": true      
            }
        "##;
        let x = serde_json::from_str::<Trade>(websocket_trade);
        // println!("{:?}", x);
        assert!(x.is_ok());
    }
}
