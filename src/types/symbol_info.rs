use std::collections::HashMap;

use crate::client::string_to_f64;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{account::Permission, order::OrderType, rate_limit::RateLimit};

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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

    #[serde(rename = "TRAILING_DELTA", rename_all = "camelCase")]
    TrailingDelta {
        min_trailing_above_delta: u64,
        max_trailing_above_delta: u64,
        min_trailing_below_delta: u64,
        max_trailing_below_delta: u64,
    },

    #[serde(rename = "PERCENT_PRICE_BY_SIDE", rename_all = "camelCase")]
    PercentPriceBySide {
        #[serde(deserialize_with = "string_to_f64")]
        bid_multiplier_up: f64,
        #[serde(deserialize_with = "string_to_f64")]
        bid_multiplier_down: f64,
        #[serde(deserialize_with = "string_to_f64")]
        ask_multiplier_up: f64,
        #[serde(deserialize_with = "string_to_f64")]
        ask_multiplier_down: f64,
        avg_price_mins: u32,
    },

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInfo {
    pub symbol: String,
    pub status: SymbolStatus,
    /// 例如对于BTCUSDT来说，base_asset是BTC
    pub base_asset: String,
    pub base_asset_precision: u8,
    /// 例如对于BTCUSDT来说，quote_asset是USDT
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
    #[serde(default)]
    pub is_spot_trading_allowed: bool,
    #[serde(default)]
    pub is_margin_trading_allowed: bool,
    pub filters: Vec<SymbolFilter>,
    /// 未来会替代is_(spot|margin)_trading_allowed字段
    pub permissions: Vec<Permission>,
    /// 未来可能添加新字段，全部放入此处，避免直接报错
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
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
