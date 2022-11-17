use crate::client::string_to_f64;
use serde::Deserialize;

/// 买盘卖盘信息(最高买单信息和最低卖单信息)
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BookTickers {
    BookTicker(BookTicker),
    BookTickers(Vec<BookTicker>),
}

/// 买盘卖盘信息(最高买单信息和最低卖单信息)
#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum WrapBookTicker {
    RestBookTicker(RestBookTicker),
    WebSocketBookTicker(WebSocketBookTicker),
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FullTickers {
    Hr24FullTicker(FullTicker),
    Hr24FullTickers(Vec<FullTicker>),
}

/// 24小时内价格变动信息
#[derive(Debug, Deserialize)]
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
    pub first_id: i64,
    /// 24小时区间的最后一笔成交ID(即最近一次的成交ID)
    pub last_id: i64,
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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum WrapFullTicker {
    RestFullTicker(RestFullHr24),
    WebSocketFullTicker(WebSocketFullTicker),
}

/// 24小时内价格变动信息(Rest接口)
#[derive(Debug, Deserialize)]
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
    first_id: i64,

    /// 24小时区间的最后一笔成交ID(即最近一次的成交ID)
    last_id: i64,

    /// 24小时区间的成交笔数
    count: u64,
}

/// 24小时内价格变动信息(WebSocket接口)
#[derive(Debug, Deserialize)]
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
    first_id: i64,

    /// 24小时区间的最后一笔成交ID(即最近一次的成交ID)
    #[serde(rename = "L")]
    last_id: i64,

    /// 24小时区间的成交笔数
    #[serde(rename = "n")]
    count: u64,
}

/// 按Symbol或全市场的精简Ticker(可能是单个元素，可能是Vec容器)
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MiniTickers {
    MiniTicker(MiniTicker),
    MiniTickers(Vec<MiniTicker>),
}

/// 按Symbol或全市场的精简Ticker
#[derive(Debug, Deserialize)]
pub struct MiniTicker {
    /// 交易对
    #[serde(rename = "s")]
    pub symbol: String,
    /// 最新成交价格
    #[serde(rename = "c", deserialize_with = "string_to_f64")]
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
    use crate::types::ticker::{BookTickers, FullTickers};

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
