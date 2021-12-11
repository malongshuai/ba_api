use crate::{client::string_to_f64, types::order::OrderType};
use serde::{Deserialize, Serialize};

use super::account::RateLimit;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    pub server_time: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SymbolStatus {
    /// 交易前
    PreTrading,
    /// 交易中
    Trading,
    /// 交易后
    PostTrading,
    EndOfDay,
    Halt,
    AuctionMatch,
    Break,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "filterType")]
pub enum SymbolFilter {
    #[serde(rename = "PRICE_FILTER", rename_all = "camelCase")]
    PriceFilter {
        #[serde(deserialize_with = "string_to_f64")]
        min_price: f64,
        #[serde(deserialize_with = "string_to_f64")]
        max_price: f64,
        #[serde(deserialize_with = "string_to_f64")]
        tick_size: f64,
    },

    #[serde(rename = "PERCENT_PRICE", rename_all = "camelCase")]
    PercentPrice {
        #[serde(deserialize_with = "string_to_f64")]
        multiplier_up: f64,
        #[serde(deserialize_with = "string_to_f64")]
        multiplier_down: f64,
        avg_price_mins: u64,
    },

    #[serde(rename = "LOT_SIZE", rename_all = "camelCase")]
    LotSize {
        #[serde(deserialize_with = "string_to_f64")]
        min_qty: f64,
        #[serde(deserialize_with = "string_to_f64")]
        max_qty: f64,
        #[serde(deserialize_with = "string_to_f64")]
        step_size: f64,
    },

    #[serde(rename = "MIN_NOTIONAL", rename_all = "camelCase")]
    MinNotional {
        #[serde(deserialize_with = "string_to_f64")]
        min_notional: f64,
        apply_to_market: bool,
        avg_price_mins: u64,
    },

    #[serde(rename = "ICEBERG_PARTS")]
    IcebergParts { limit: u64 },

    #[serde(rename = "MARKET_LOT_SIZE", rename_all = "camelCase")]
    MarketLotSize {
        #[serde(deserialize_with = "string_to_f64")]
        min_qty: f64,
        #[serde(deserialize_with = "string_to_f64")]
        max_qty: f64,
        #[serde(deserialize_with = "string_to_f64")]
        step_size: f64,
    },

    #[serde(rename = "MAX_NUM_ORDERS", rename_all = "camelCase")]
    MaxNumOrders { max_num_orders: u64 },

    #[serde(rename = "MAX_NUM_ALGO_ORDERS", rename_all = "camelCase")]
    MaxNumAlgoOrders { max_num_algo_orders: u64 },

    #[serde(rename = "MAX_NUM_ICEBERG_ORDERS", rename_all = "camelCase")]
    MaxNumIcebergOrders { max_num_iceberg_orders: u64 },

    #[serde(rename = "MAX_POSITION", rename_all = "camelCase")]
    MaxPosition {
        #[serde(deserialize_with = "string_to_f64")]
        max_position: f64,
    },
}

/// 交易权限
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Permission {
    Spot,
    Margin,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInfo {
    pub symbol: String,
    pub status: SymbolStatus,
    pub base_asset: String,
    pub base_asset_precision: u8,
    pub quote_asset: String,
    pub quote_precision: u8,
    /// 替代quote_precision字段
    pub quote_asset_precision: u8,
    pub base_commission_precision: u8,
    pub quote_commission_precision: u8,
    pub order_types: Vec<OrderType>,
    pub iceberg_allowed: bool,
    pub oco_allowed: bool,
    pub quote_order_qty_market_allowed: bool,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub filters: Vec<SymbolFilter>,
    /// 目前只有两种值，spot/margin，未来会替代is_(spot|margin)_trading_allowed字段
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
    pub timezone: String,
    pub server_time: u64,
    pub rate_limits: Vec<RateLimit>,
    pub exchange_filters: Vec<String>,
    pub symbols: Vec<SymbolInfo>,
}

/// 买盘
#[derive(Debug, Deserialize, Serialize)]
pub struct BID {
    /// 挂单价
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,
    /// 此价位上的挂单数量
    #[serde(deserialize_with = "string_to_f64")]
    pub amount: f64,
}

/// 卖盘
#[derive(Debug, Deserialize, Serialize)]
pub struct ASK {
    /// 挂单价
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,
    /// 此价位上的挂单数量
    #[serde(deserialize_with = "string_to_f64")]
    pub amount: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Depth {
    pub last_update_id: u64,
    /// 买盘信息
    pub bids: Vec<BID>,
    /// 卖盘信息
    pub asks: Vec<ASK>,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct AvgPrice {
    pub mins: u64,
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,
}

/// 最新价格
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Prices {
    P(Price),
    Prices(Vec<Price>),
}

/// 最新价格
#[derive(Debug, Deserialize, Serialize)]
pub struct Price {
    pub symbol: String,
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BookTickers {
    BookTicker(BookTicker),
    BookTickers(Vec<BookTicker>),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(from = "WrapBookTicker")]
pub struct BookTicker {
    pub symbol: String,
    /// 最高买价
    pub bid_price: f64,
    /// 最高买价的挂单数量
    pub bid_qty: f64,
    /// 最低卖价
    pub ask_price: f64,
    /// 最低卖价的挂单数量
    pub ask_qty: f64,
}

impl From<WrapBookTicker> for BookTicker {
    fn from(book_ticker: WrapBookTicker) -> Self {
        match book_ticker {
            WrapBookTicker::RestBookTicker(data) => Self {
                symbol: data.symbol,
                bid_price: data.bid_price,
                bid_qty: data.bid_qty,
                ask_price: data.ask_price,
                ask_qty: data.ask_qty,
            },
            WrapBookTicker::WebSocketBookTicker(data) => Self {
                symbol: data.symbol,
                bid_price: data.bid_price,
                bid_qty: data.bid_qty,
                ask_price: data.ask_price,
                ask_qty: data.ask_qty,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum WrapBookTicker {
    RestBookTicker(RestBookTicker),
    WebSocketBookTicker(WebSocketBookTicker),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RestBookTicker {
    symbol: String,
    /// 最高买价
    #[serde(deserialize_with = "string_to_f64")]
    bid_price: f64,
    /// 最高买价的挂单数量
    #[serde(deserialize_with = "string_to_f64")]
    bid_qty: f64,

    /// 最低卖价
    #[serde(deserialize_with = "string_to_f64")]
    ask_price: f64,
    /// 最低卖价的挂单数量
    #[serde(deserialize_with = "string_to_f64")]
    ask_qty: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct WebSocketBookTicker {
    #[serde(rename = "s")]
    symbol: String,
    /// 最高买价
    #[serde(rename = "b")]
    #[serde(deserialize_with = "string_to_f64")]
    bid_price: f64,
    /// 最高买价的挂单数量
    #[serde(rename = "B")]
    #[serde(deserialize_with = "string_to_f64")]
    bid_qty: f64,

    /// 最低卖价
    #[serde(rename = "a")]
    #[serde(deserialize_with = "string_to_f64")]
    ask_price: f64,
    /// 最低卖价的挂单数量
    #[serde(rename = "A")]
    #[serde(deserialize_with = "string_to_f64")]
    ask_qty: f64,
}

/// 24小时内价格变动信息(可能是单个元素，可能是Vec容器)
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FullTickers {
    Hr24FullTicker(FullTicker),
    Hr24FullTickers(Vec<FullTicker>),
}

/// 24小时内价格变动信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(from = "WrapFullTicker")]
pub struct FullTicker {
    /// 交易对
    pub symbol: String,
    /// 24小时价格变化
    pub price_change: f64,
    /// 24小时价格变化百分比
    pub price_change_percent: f64,
    /// 24小时平均价格
    pub avg_price: f64,
    /// 24小时前的最后一个成交价格(也即上一个24小时的收盘价)
    pub prev_close: f64,
    /// 当前最新价格(实时价格)
    pub close: f64,
    /// 最后一次(即上一次)成交的数量
    pub last_qty: f64,
    /// 当前最高买单价
    pub bid_price: f64,
    /// 当前最高买单数量
    pub bid_qty: f64,
    /// 当前最低卖单价
    pub ask_price: f64,
    /// 当前最低卖单数量
    pub ask_qty: f64,
    /// 24小时区间的开盘价
    pub open: f64,
    /// 24小时区间的最高价
    pub high: f64,
    pub low: f64,
    /// 24小时区间的成交量
    pub amount: f64,
    /// 24小时区间的成交额
    pub vol: f64,
    /// 统计开始时间(即24小时前)
    pub open_time: u64,
    /// 统计结束时间(即当前时间)
    pub close_time: u64,
    /// 24小时区间的第一笔成交ID
    pub first_id: u64,
    /// 24小时区间的最后一笔成交ID(即最近一次的成交ID)
    pub last_id: u64,
    /// 24小时区间的成交笔数
    pub count: u64,
}

impl From<WrapFullTicker> for FullTicker {
    fn from(ticker: WrapFullTicker) -> Self {
        match ticker {
            WrapFullTicker::RestFullTicker(data) => Self {
                symbol: data.symbol,
                price_change: data.price_change,
                price_change_percent: data.price_change_percent,
                avg_price: data.avg_price,
                prev_close: data.prev_close,
                close: data.close,
                last_qty: data.last_qty,
                bid_price: data.bid_price,
                bid_qty: data.bid_qty,
                ask_price: data.ask_price,
                ask_qty: data.ask_qty,
                open: data.open,
                high: data.high,
                low: data.low,
                amount: data.amount,
                vol: data.vol,
                open_time: data.open_time,
                close_time: data.close_time,
                first_id: data.first_id,
                last_id: data.last_id,
                count: data.count,
            },
            WrapFullTicker::WebSocketFullTicker(data) => Self {
                symbol: data.symbol,
                price_change: data.price_change,
                price_change_percent: data.price_change_percent,
                avg_price: data.avg_price,
                prev_close: data.prev_close,
                close: data.close,
                last_qty: data.last_qty,
                bid_price: data.bid_price,
                bid_qty: data.bid_qty,
                ask_price: data.ask_price,
                ask_qty: data.ask_qty,
                open: data.open,
                high: data.high,
                low: data.low,
                amount: data.amount,
                vol: data.vol,
                open_time: data.open_time,
                close_time: data.close_time,
                first_id: data.first_id,
                last_id: data.last_id,
                count: data.count,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum WrapFullTicker {
    RestFullTicker(RestFullHr24),
    WebSocketFullTicker(WebSocketFullTicker),
}

/// 24小时内价格变动信息(Rest接口)
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RestFullHr24 {
    /// 交易对
    symbol: String,

    /// 24小时价格变化
    #[serde(deserialize_with = "string_to_f64")]
    price_change: f64,

    /// 24小时价格变化百分比
    #[serde(deserialize_with = "string_to_f64")]
    price_change_percent: f64,

    /// 24小时平均价格
    #[serde(rename = "weightedAvgPrice", deserialize_with = "string_to_f64")]
    avg_price: f64,

    /// 24小时前的最后一个成交价格(也即上一个24小时的收盘价)
    #[serde(rename = "prevClosePrice", deserialize_with = "string_to_f64")]
    prev_close: f64,

    /// 当前最新价格(实时价格)
    #[serde(rename = "lastPrice", deserialize_with = "string_to_f64")]
    close: f64,

    /// 最后一次(即上一次)成交的数量
    #[serde(deserialize_with = "string_to_f64")]
    last_qty: f64,

    /// 当前最高买单价
    #[serde(deserialize_with = "string_to_f64")]
    bid_price: f64,

    /// 当前最高买单数量
    #[serde(deserialize_with = "string_to_f64")]
    bid_qty: f64,

    /// 当前最低卖单价
    #[serde(deserialize_with = "string_to_f64")]
    ask_price: f64,

    /// 当前最低卖单数量
    #[serde(deserialize_with = "string_to_f64")]
    ask_qty: f64,

    /// 24小时区间的开盘价
    #[serde(rename = "openPrice", deserialize_with = "string_to_f64")]
    open: f64,

    /// 24小时区间的最高价
    #[serde(rename = "highPrice", deserialize_with = "string_to_f64")]
    high: f64,

    #[serde(rename = "lowPrice", deserialize_with = "string_to_f64")]
    low: f64,

    /// 24小时区间的成交量
    #[serde(rename = "volume", deserialize_with = "string_to_f64")]
    amount: f64,

    /// 24小时区间的成交额
    #[serde(rename = "quoteVolume", deserialize_with = "string_to_f64")]
    vol: f64,

    /// 统计开始时间(即24小时前)
    open_time: u64,

    /// 统计结束时间(即当前时间)
    close_time: u64,

    /// 24小时区间的第一笔成交ID
    first_id: u64,

    /// 24小时区间的最后一笔成交ID(即最近一次的成交ID)
    last_id: u64,

    /// 24小时区间的成交笔数
    count: u64,
}

/// 24小时内价格变动信息(WebSocket接口)
#[derive(Debug, Deserialize, Serialize)]
struct WebSocketFullTicker {
    /// 交易对
    #[serde(rename = "s")]
    symbol: String,

    /// 24小时价格变化
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "p")]
    price_change: f64,

    /// 24小时价格变化百分比
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "P")]
    price_change_percent: f64,

    /// 24小时平均价格
    #[serde(rename = "w")]
    #[serde(deserialize_with = "string_to_f64")]
    avg_price: f64,

    /// 24小时前的最后一个成交价格(也即上一个24小时的收盘价)
    #[serde(rename = "x")]
    #[serde(deserialize_with = "string_to_f64")]
    prev_close: f64,

    /// 当前最新价格(实时价格)
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "c")]
    close: f64,

    /// 最后一次(即上一次)成交的数量
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "Q")]
    last_qty: f64,

    /// 当前最高买单价
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "b")]
    bid_price: f64,

    /// 当前最高买单数量
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "B")]
    bid_qty: f64,

    /// 当前最低卖单价
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "a")]
    ask_price: f64,

    /// 当前最低卖单数量
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "A")]
    ask_qty: f64,

    /// 24小时区间的开盘价
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "o")]
    open: f64,

    /// 24小时区间的最高价
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "h")]
    high: f64,

    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "l")]
    low: f64,

    /// 24小时区间的成交量
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "v")]
    amount: f64,

    /// 24小时区间的成交额
    #[serde(deserialize_with = "string_to_f64")]
    #[serde(rename = "q")]
    vol: f64,

    /// 统计开始时间(即24小时前)
    #[serde(rename = "O")]
    open_time: u64,

    /// 统计结束时间(即当前时间)
    #[serde(rename = "C")]
    close_time: u64,

    /// 24小时区间的第一笔成交ID
    #[serde(rename = "F")]
    first_id: u64,

    /// 24小时区间的最后一笔成交ID(即最近一次的成交ID)
    #[serde(rename = "L")]
    last_id: u64,

    /// 24小时区间的成交笔数
    #[serde(rename = "n")]
    count: u64,
}

/// 按Symbol或全市场的精简Ticker(可能是单个元素，可能是Vec容器)
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MiniTickers {
    MiniTicker(MiniTicker),
    MiniTickers(Vec<MiniTicker>),
}

/// 按Symbol或全市场的精简Ticker
#[derive(Debug, Serialize, Deserialize)]
pub struct MiniTicker {
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    /// 最新成交价格
    #[serde(rename = "t", deserialize_with = "string_to_f64")]
    pub close: f64,
    /// 24小时前开始第一笔成交价格
    #[serde(rename = "o", deserialize_with = "string_to_f64")]
    pub open: f64,
    /// 24小时内最高成交价
    #[serde(rename = "h", deserialize_with = "string_to_f64")]
    pub high: f64,
    /// 24小时内最低成交价
    #[serde(rename = "l", deserialize_with = "string_to_f64")]
    pub low: f64,
    /// 成交量
    #[serde(rename = "v", deserialize_with = "string_to_f64")]
    pub amount: f64,
    /// 成交额
    #[serde(rename = "q", deserialize_with = "string_to_f64")]
    pub vol: f64,
}

#[cfg(test)]
mod test {
    use super::*;

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

    #[test]
    fn test_rest_bookticker() {
        let str = r##"
            [
              {
                "symbol": "LTCBTC",
                "bidPrice": "4.00000000",
                "bidQty": "431.00000000",
                "askPrice": "4.00000200",
                "askQty": "9.00000000"
              },
              {
                "symbol": "ETHBTC",
                "bidPrice": "0.07946700",
                "bidQty": "9.00000000",
                "askPrice": "100000.00000000",
                "askQty": "1000.00000000"
              }
            ]
        "##;
        let x = serde_json::from_str::<BookTickers>(str);
        // println!("{:?}", x);
        assert!(x.is_ok());
    }

    #[test]
    fn test_ws_bookticker() {
        let str = r##"
            {
              "u":400900217,     
              "s":"BNBUSDT",     
              "b":"25.35190000", 
              "B":"31.21000000", 
              "a":"25.36520000", 
              "A":"40.66000000"  
            }
        "##;
        let x = serde_json::from_str::<BookTickers>(str);
        // println!("{:?}", x);
        assert!(x.is_ok());
    }

    #[test]
    fn test_rest_full_ticker() {
        let rest_full_ticker = r##"
          [
            {
              "symbol": "BNBBTC",
              "priceChange": "-94.99999800",
              "priceChangePercent": "-95.960",
              "weightedAvgPrice": "0.29628482",
              "prevClosePrice": "0.10002000",
              "lastPrice": "4.00000200",
              "lastQty": "200.00000000",
              "bidPrice": "4.00000000",
              "bidQty": "100.00000000",
              "askPrice": "4.00000200",
              "askQty": "100.00000000",
              "openPrice": "99.00000000",
              "highPrice": "100.00000000",
              "lowPrice": "0.10000000",
              "volume": "8913.30000000",
              "quoteVolume": "15.30000000",
              "openTime": 1499783499040,
              "closeTime": 1499869899040,
              "firstId": 28385,  
              "lastId": 28460,   
              "count": 76      
            }
          ]
        "##;
        let x = serde_json::from_str::<FullTickers>(rest_full_ticker);
        // println!("{:?}", x);
        assert!(x.is_ok());
    }
    
    #[test]
    fn test_ws_full_ticker() {
        let websocket_full_ticker = r##"
          {
            "e": "24hrTicker",
            "E": 123456789,   
            "s": "BNBBTC",    
            "p": "0.0015",    
            "P": "250.00",    
            "w": "0.0018",    
            "x": "0.0009",    
            "c": "0.0025",    
            "Q": "10",        
            "b": "0.0024",    
            "B": "10",        
            "a": "0.0026",    
            "A": "100",       
            "o": "0.0010",    
            "h": "0.0025",    
            "l": "0.0010",    
            "v": "10000",     
            "q": "18",        
            "O": 0,           
            "C": 86400000,    
            "F": 0,           
            "L": 18150,       
            "n": 18151        
          }
        "##;
        let x = serde_json::from_str::<FullTickers>(websocket_full_ticker);
        // println!("{:?}", x);
        assert!(x.is_ok());
    }
}
