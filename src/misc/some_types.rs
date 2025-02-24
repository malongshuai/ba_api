use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// 如果确实有警告，警告内容存放在data字段(WarnMessage类型)的content字段中
#[derive(Debug, Deserialize)]
pub(super) struct WarnInfo {
    pub(super) code: String,
    pub(super) success: bool,
    pub(super) message: Option<String>,
    pub(super) messageDetail: Option<String>,
    pub(super) data: Option<WarnMessage>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct WarnMessage {
    pub(super) content: Option<String>,
    pub(super) title: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

/// 如果确实要下架，下架信息存放在data字段(TradingPairInfo类型)中
#[derive(Debug, Deserialize)]
pub(super) struct TradingPairInfoWrap {
    pub(super) code: String,
    pub(super) success: bool,
    pub(super) message: Option<String>,
    pub(super) messageDetail: Option<String>,
    pub(super) data: Option<TradingPairInfo>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

/// 表示一个交易对的信息
#[derive(Deserialize, Debug)]
pub struct TradingPairInfo {
    /// 交易对 (Symbol)，如 OAXUSDT
    #[serde(alias = "s")]
    pub symbol: String,

    /// 状态 (Status)，正在交易的状态为"TRADING"
    #[serde(alias = "st")]
    pub status: String,

    /// 基础资产符号 (Base Asset)，如 OAXUSDT 的 OAX
    #[serde(alias = "b")]
    pub base_asset: String,

    /// 报价资产符号 (Quote Asset)，如 OAXUSDT 的 USDT
    #[serde(alias = "q")]
    pub quote_asset: String,

    // /// 基础资产精度 (Base Asset Precision)
    // #[serde(alias = "ba")]
    // pub base_asset_precision: Option<String>,
    // /// 报价资产精度 (Quote Asset Precision)
    // #[serde(alias = "qa")]
    // pub quote_asset_precision: Option<String>,
    /// 标签 (Tags)，每个交易对都带有标签，比如种子标签、观察标签、Layer2标签等
    #[serde(alias = "tags")]
    pub tags: Vec<String>,

    /// 是否计划下架 (Planned to be Delisted)，true表示已经在下架计划中，即将要在不远的未来下架
    #[serde(alias = "pom")]
    pub to_delist: bool,

    /// 计划下架时间 (Planned Delisting Time)，如果计划下架，值为具体的下架时间点，为毫秒级Epoch，否则为None
    #[serde(alias = "pomt")]
    pub delist_time: Option<u64>,

    // /// 最小交易单位 (Lot Size)
    // #[serde(alias = "i")]
    // pub lot_size: String,
    // /// 价格精度 (Tick Size)
    // #[serde(alias = "ts")]
    // pub tick_size: String,
    // /// 流通供应量 (Circulating Supply)
    // #[serde(alias = "cs")]
    // pub circulating_supply: Option<u64>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}
