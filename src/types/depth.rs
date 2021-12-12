use crate::client::string_to_f64;
use serde::{Deserialize, Serialize};

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

/// 深度信息
///
/// 从Rest接口获得的深度数据以及从WebSocket获得的分档深度数据，symbol为空，first_update_id为0  
/// 从WebSocket接口获得的增量深度数据，symbol和first_update_id均为有效值
#[derive(Debug, Deserialize, Serialize)]
#[serde(from = "WrapDepth")]
pub struct Depth {
    pub symbol: String,
    /// 上次推送之后，本次推送的新增深度数据中的第一个 update Id
    pub first_update_id: u64,
    /// 上次推送之后，本次推送的新增深度数据中的最后一个 update Id  
    /// 也即当前最实时的一个update id
    pub last_update_id: u64,
    /// 买盘信息
    pub bids: Vec<BID>,
    /// 卖盘信息
    pub asks: Vec<ASK>,
}

impl From<WrapDepth> for Depth {
    fn from(depth: WrapDepth) -> Self {
        match depth {
            WrapDepth::RestDepth(data) => Self {
                symbol: String::new(),
                first_update_id: 0,
                last_update_id: data.last_update_id,
                bids: data.bids,
                asks: data.asks,
            },
            WrapDepth::WebSocketDepth(data) => Self {
                symbol: data.symbol,
                first_update_id: data.first_update_id,
                last_update_id: data.last_update_id,
                bids: data.bids,
                asks: data.asks,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum WrapDepth {
    RestDepth(RestDepth),
    WebSocketDepth(WebSocketDepth),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RestDepth {
    last_update_id: u64,
    /// 买盘信息
    bids: Vec<BID>,
    /// 卖盘信息
    asks: Vec<ASK>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WebSocketDepth {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "U")]
    first_update_id: u64,
    #[serde(rename = "u")]
    last_update_id: u64,
    /// 买盘信息
    #[serde(rename = "b")]
    bids: Vec<BID>,
    /// 卖盘信息
    #[serde(rename = "a")]
    asks: Vec<ASK>,
}

#[cfg(test)]
mod test {
    use crate::types::depth::Depth;

    #[test]
    fn test_rest_depth() {
        let str = r##"
            {
              "lastUpdateId": 160,  
              "bids": [             
                [
                  "0.0024",         
                  "10"              
                ]
              ],
              "asks": [             
                [
                  "0.0026",         
                  "100"             
                ]
              ]
            }
        "##;
        let x = serde_json::from_str::<Depth>(str);
        // println!("{:?}", x);
        assert!(x.is_ok());
    }

    #[test]
    fn test_ws_depth() {
        let str = r##"
            {
              "e": "depthUpdate", 
              "E": 123456789,     
              "s": "BNBBTC",      
              "U": 157,           
              "u": 160,           
              "b": [              
                [
                  "0.0024",       
                  "10"            
                ]
              ],
              "a": [              
                [
                  "0.0026",       
                  "100"           
                ]
              ]
            }
        "##;
        let x = serde_json::from_str::<Depth>(str);
        // println!("{:?}", x);
        assert!(x.is_ok());
    }
}
