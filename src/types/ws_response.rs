use super::account::Balances;
use super::depth::Depth;
use super::order::{AggTrade, OrderUpdate, Trade};
use super::ticker::{BookTickers, FullTickers, MiniTickers};
use crate::client::string_to_f64;
use crate::KLine;
use serde::{Deserialize, Deserializer, Serialize};

#[allow(dead_code)]
fn flatten_balance<'de, D>(deserializer: D) -> Result<Balances, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct WrapBalance {
        #[serde(rename = "B")]
        balances: Balances,
    }

    Ok(WrapBalance::deserialize(deserializer)?.balances)
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WsBalances {
    #[serde(rename(deserialize = "u"))]
    pub event_time: u64,
    #[serde(rename(deserialize = "B"))]
    pub balances: Balances,
}

impl WsBalances {
    pub fn new() -> WsBalances {
        Self::default()
    }
}

/// 余额更新信息(例如充值、提现、划转)
#[derive(Debug, Deserialize, Serialize)]
pub struct BalanceUpdate {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "d", deserialize_with = "string_to_f64")]
    pub delta: f64,
    #[serde(rename = "T")]
    pub clear_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "e")]
pub enum WebSocketAccount {
    /// 余额更新(例如充值、提现、划转)
    #[serde(rename = "balanceUpdate")]
    BalanceUpdate(BalanceUpdate),

    /// 账户余额更新(例如订单成交)
    #[serde(rename = "outboundAccountPosition")]
    // #[serde(deserialize_with = "flatten_balance")]
    // OutboundAccountPosition(Balances),
    OutboundAccountPosition(WsBalances),

    /// 订单更新
    #[serde(rename = "executionReport")]
    ExecutionReport(Box<OrderUpdate>),
}

/// 订阅WebSocket时响应的数据类型
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response {
    /// 和账户有关的推送数据，包括余额更新、订单更新、资产更新，必须放在Trade字段的的前面
    Account(WebSocketAccount),

    /// 逐笔交易，必须放在Account字段的的后面
    Trade(Trade),

    /// 归集交易的推送数据
    AggTrade(AggTrade),

    /// K线数据
    KLine(KLine),

    /// 按Symbol的或全市场的完整的Ticker，必须放在MiniTickers字段的的前面
    FullTickers(FullTickers),

    /// 按Symbol的或全市场的精简的Ticker，必须放在FullTickers字段的的后面
    MiniTickers(MiniTickers),

    /// 按Symbol的或全市场的完整的最优挂单信息(BookTicker)
    BookTickers(BookTickers),

    /// 按Symbol的有限档深度和增量深度信息  
    /// 有限档深度信息中的symbol字段为空，且first_update_id字段为0  
    /// 增量深度信息中，这两个字段均有有效值  
    Depth(Depth),
}

/// 订阅WebSocket组合流时响应的数据类型
#[derive(Debug, Serialize, Deserialize)]
pub struct WsResponse {
    pub stream: String,
    pub data: Response,
}

#[cfg(test)]
mod account_test {
    use super::*;
    #[test]
    fn outbound_account_position() {
        let s = r##"
            {
              "e": "outboundAccountPosition",
              "E": 1564034571105,
              "u": 1564034571073,
              "B": [
                {
                  "a": "ETH",
                  "f": "10000.000000",
                  "l": "0.000000"
                },
                {
                  "a": "BTC",
                  "f": "2.00",
                  "l": "1.445"
                }
              ]
            }
        "##;
        let x = serde_json::from_str::<WebSocketAccount>(s);
        // println!("{:?}", x);
        assert!(x.is_ok());
    }

    #[test]
    fn balance_update() {
        let s = r##"
            {
              "e": "balanceUpdate",
              "E": 1573200697110,  
              "a": "ABC",          
              "d": "100.00000000", 
              "T": 1573200697068   
            }
        "##;
        let x = serde_json::from_str::<WebSocketAccount>(s);
        // println!("{:?}", x);
        assert!(x.is_ok());
    }

    #[test]
    fn execution_report() {
        let s = r##"
            {
              "e": "executionReport",       
              "E": 1499405658658,           
              "s": "ETHBTC",                
              "c": "mUvoqJxFIILMdfAW5iGSOW",
              "S": "BUY",                   
              "o": "LIMIT",                 
              "f": "GTC",                   
              "q": "1.00000000",            
              "p": "0.10264410",            
              "P": "0.00000000",            
              "F": "0.00000000",            
              "g": -1,                      
              "C": "",                      
              "x": "NEW",                   
              "X": "NEW",                   
              "r": "NONE",                  
              "i": 4293153,                 
              "l": "0.00000000",            
              "z": "0.00000000",            
              "L": "0.00000000",            
              "n": "0",                     
              "N": null,                    
              "T": 1499405658657,           
              "t": -1,                      
              "I": 8641984,                 
              "w": true,                    
              "m": false,                   
              "M": false,                   
              "O": 1499405658657,           
              "Z": "0.00000000",            
              "Y": "0.00000000",            
              "Q": "0.00000000"             
            }
        "##;
        let x = serde_json::from_str::<WebSocketAccount>(s);
        // println!("{:?}", x);
        assert!(x.is_ok());
    }
}
