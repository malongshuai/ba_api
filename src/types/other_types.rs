use crate::client::string_to_f64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    pub server_time: u64,
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

