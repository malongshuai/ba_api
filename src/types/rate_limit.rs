use serde::{Deserialize, Serialize};



#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitType {
    RequestWeight,
    Orders,
    RawRequests,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitInterVal {
    Second,
    Minute,
    Day,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: RateLimitType,
    pub interval: RateLimitInterVal,
    pub interval_num: u32,
    pub limit: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitInfo {
    pub rate_limit_type: RateLimitType,
    pub interval: RateLimitInterVal,
    pub interval_num: u32,
    pub limit: u32,
    pub count: u32,
}
