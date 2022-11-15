use serde::{Deserialize, Deserializer};

/// REST客户端连接
pub mod rest;

/// WebSocket客户端连接
pub mod websocket;

/// Rest请求的参数和签名鉴权类型
///
/// 所有请求都需要实现Serialize和Param Trait，
/// 如果请求的参数为空，则定义为空的Struct并实现这两个Trait
pub mod params;

/// [行情接口](rest/struct.RestConn.html#impl-1)，币安API Doc行情接口下的方法都在此
pub mod market_data;

/// [现货交易接口](rest/struct.RestConn.html#impl-2)，币安API Doc现货账户和现货交易接口下的方法都在此
pub mod spot_account_trade;

pub mod sub_account;
/// 钱包相关接口
pub mod wallet;

pub use rest::*;
pub use websocket::*;

/// String转换为f64
#[allow(dead_code)]
pub(crate) fn string_to_f641<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Str<'a>(&'a str);

    match Str::deserialize(deserializer)? {
        Str(a) if a.is_empty() => Ok(0.0),
        Str(a) => a.parse::<f64>().map_err(|_| {
            serde::de::Error::invalid_type(serde::de::Unexpected::Str(a), &"f64 string")
        }),
    }
}

/// String转换为f64，除了str -> f64, 还可同时处理f64 -> f64
pub(crate) fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum M<'a> {
        F64(f64),
        Str(&'a str),
    }

    match M::deserialize(deserializer)? {
        M::F64(f) => Ok(f),
        M::Str(s) => {
            if s.is_empty() {
                Ok(0.0)
            } else {
                s.parse::<f64>().map_err(|_| {
                    serde::de::Error::invalid_type(serde::de::Unexpected::Str(s), &"f64 string")
                })
            }
        }
    }
}

/// Option<String>转换为Option<f64>
pub(crate) fn option_string_to_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    // #[derive(Deserialize)]
    // struct Str<'a>(&'a str);

    match Option::deserialize(deserializer)? {
        None => Ok(None),
        Some::<String>(str) => {
            if str.is_empty() {
                Ok(Some(0.0))
            } else {
                let parse_res = str.parse::<f64>().map_err(|_| {
                    serde::de::Error::invalid_type(serde::de::Unexpected::Str(&str), &"f64 string")
                })?;
                Ok(Some(parse_res))
            }
        }
    }
}
