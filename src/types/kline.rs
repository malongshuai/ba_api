use std::fmt::Display;

use crate::client::string_to_f64;
use serde::{Deserialize, Serialize};

use super::kline_interval::KLineInterval;

/// K线数组，Vec<KLine>
pub type KLines = Vec<KLine>;

/// K线
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(from = "WrapKLine")]
pub struct KLine {
    /// 交易对
    pub symbol: String,
    /// K线间隔
    pub interval: KLineInterval,
    /// 开盘时间
    pub epoch: u64,
    /// 收盘时间
    pub close_epoch: u64,
    /// 该K线是否已经收盘
    pub finish: bool,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量
    pub amount: f64,
    /// 成交额
    pub vol: f64,
    /// 成交笔数
    pub count: u64,
}

impl Display for KLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum WrapKLine {
    Rest(RestKLine),
    WebSocket(WebSocketKLine),
    Raw(RawKLine),
}

impl From<WrapKLine> for KLine {
    fn from(wk: WrapKLine) -> Self {
        match wk {
            WrapKLine::Rest(data) => Self {
                symbol: String::new(),
                epoch: data.epoch,
                close_epoch: data.close_epoch,
                high: data.high,
                close: data.close,
                low: data.low,
                open: data.open,
                count: data.count,
                amount: data.amount,
                vol: data.vol,
                finish: true,
                interval: KLineInterval::Min1,
            },
            WrapKLine::WebSocket(data) => Self {
                symbol: data.kline.symbol,
                epoch: data.kline.epoch,
                close_epoch: data.kline.close_epoch,
                high: data.kline.high,
                close: data.kline.close,
                low: data.kline.low,
                open: data.kline.open,
                count: data.kline.count,
                amount: data.kline.amount,
                vol: data.kline.vol,
                finish: data.kline.finish,
                interval: data.kline.interval,
            },
            WrapKLine::Raw(data) => Self {
                symbol: data.symbol,
                epoch: data.epoch,
                close_epoch: data.close_epoch,
                high: data.high,
                close: data.close,
                low: data.low,
                open: data.open,
                count: data.count,
                amount: data.amount,
                vol: data.vol,
                finish: data.finish,
                interval: data.interval,
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawKLine {
    /// 交易对
    pub symbol: String,
    /// K线间隔
    pub interval: KLineInterval,
    /// 开盘时间
    pub epoch: u64,
    /// 收盘时间
    pub close_epoch: u64,
    /// 该K线是否已经收盘
    pub finish: bool,
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 成交量
    pub amount: f64,
    /// 成交额
    pub vol: f64,
    /// 成交笔数
    pub count: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct RestKLine {
    epoch: u64,
    #[serde(deserialize_with = "string_to_f64")]
    open: f64,
    #[serde(deserialize_with = "string_to_f64")]
    high: f64,
    #[serde(deserialize_with = "string_to_f64")]
    low: f64,
    #[serde(deserialize_with = "string_to_f64")]
    close: f64,
    #[serde(deserialize_with = "string_to_f64")]
    amount: f64,
    close_epoch: u64,
    #[serde(deserialize_with = "string_to_f64")]
    vol: f64,
    count: u64,
    #[serde(deserialize_with = "string_to_f64")]
    buy_amount: f64,
    #[serde(deserialize_with = "string_to_f64")]
    buy_vol: f64,
    #[serde(deserialize_with = "string_to_f64")]
    ignore: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct WebSocketRawKLine {
    /// 开盘时间
    #[serde(rename = "t")]
    epoch: u64,
    /// 收盘时间
    #[serde(rename = "T")]
    close_epoch: u64,
    /// 交易对(BNBBTC)
    #[serde(rename = "s")]
    symbol: String,
    /// K线时间间隔
    #[serde(rename = "i")]
    interval: KLineInterval,
    /// 最高价
    #[serde(rename = "h", deserialize_with = "string_to_f64")]
    high: f64,
    /// 开盘价
    #[serde(rename = "o", deserialize_with = "string_to_f64")]
    open: f64,
    /// 收盘价
    #[serde(rename = "c", deserialize_with = "string_to_f64")]
    close: f64,
    /// 最低价
    #[serde(rename = "l", deserialize_with = "string_to_f64")]
    low: f64,
    /// 成交量
    #[serde(rename = "v", deserialize_with = "string_to_f64")]
    amount: f64,
    /// 成交笔数
    #[serde(rename = "n")]
    count: u64,
    /// 成交额
    #[serde(rename = "q", deserialize_with = "string_to_f64")]
    vol: f64,
    /// 本K线是否完结
    #[serde(rename = "x")]
    finish: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct WebSocketKLine {
    #[serde(rename = "k")]
    kline: WebSocketRawKLine,
}

#[cfg(test)]
mod kline_test {
    use super::*;

    #[test]
    fn rest_kline() {
        let str = r##"
          [
            1499040000000, 
            "0.01634790", 
            "0.80000000",
            "0.01575800",
            "0.01577100",
            "148976.11427815",
            1499644799999,    
            "2434.19055334",  
            308,
            "1756.87402397",  
            "28.46694368",    
            "17928899.62484339"
          ]
        "##;
        let kline = serde_json::from_str::<KLine>(str);
        // println!("{:?}", kline);
        assert!(kline.is_ok());
        // let kl = serde_json::to_string(&kline.unwrap()).unwrap();
        // println!("{:?}", kl);
    }
    #[test]
    fn websocket_kline() {
        let str = r##"
        {
          "e": "kline",
          "E": 123456789,
          "s": "BTCUSDT",
          "k": {
            "t": 123400000,
            "T": 123460000,
            "s": "BNBBTC",
            "i": "1m",  
            "f": 100,     
            "L": 200,   
            "o": "0.0010", 
            "c": "0.0020", 
            "h": "0.0025", 
            "l": "0.0015", 
            "v": "1000",  
            "n": 100,      
            "x": false,     
            "q": "1.0000",  
            "V": "500",    
            "Q": "0.500",  
            "B": "123456"  
          }
        }
        "##;

        let kline = serde_json::from_str::<KLine>(str);
        // println!("{:?}", kline);
        assert!(kline.is_ok());
        // let kl = serde_json::to_string(&kline.unwrap()).unwrap();
        // println!("{:?}", kl);
    }

    #[test]
    fn raw_kline() {
      //   let str = r##"
      //       {
      //         "symbol": "BTCUSDT",
      //         "interval": "1m",
      //         "epoch":1641909600000,
      //         "close_epoch":1641909659999,
      //         "finish":true,
      //         "open":41700.74,
      //         "high":41700.74,
      //         "low":41646.69,
      //         "close":41665.83,
      //         "amount":9.03677,
      //         "vol":376549.1409579,
      //         "count":599
      //       }
      // "##;

        // let kline = serde_json::from_str::<KLine>(str);
        // println!("{:?}", kline);
        // let kl = serde_json::to_string(&kline.unwrap()).unwrap();
        // println!("{:?}", kl);
    }
}
