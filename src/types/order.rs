use crate::client::option_string_to_f64;
use crate::client::string_to_f64;
use serde::{Deserialize, Serialize};

/// 订单状态
#[derive(Debug, Deserialize, Serialize)]
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

/// 订单类型(参考<https://www.binance.com/cn/support/articles/360033779452-Types-of-Order>)
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
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

/// 订单有效方式(订单何时失效)
#[derive(Debug, Deserialize, Serialize)]
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

/// 订单信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderInfo {
    pub symbol: String,
    pub order_id: u64,
    pub client_order_id: String,

    /// OCO订单的ID，不然就是-1
    pub order_list_id: i64,

    /// 订单状态
    pub status: String, // "NEW"

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
