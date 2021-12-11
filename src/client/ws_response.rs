use crate::KLine;
use serde::{Deserialize, Serialize};

use super::rest_response::{AggTrade, BookTickers, Depth, FullTickers, MiniTickers, Trade};

/// 订阅WebSocket时响应的数据类型
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response {
    /// 归集交易的推送数据
    AggTrade(AggTrade),

    /// 逐笔交易
    Trade(Trade),

    /// K线数据
    KLine(KLine),

    /// 按Symbol的或全市场的精简的Ticker
    MiniTickers(MiniTickers),

    /// 按Symbol的或全市场的完整的Ticker
    FullTickers(FullTickers),

    /// 按Symbol的或全市场的完整的最优挂单信息(BookTicker)
    BookTickers(BookTickers),

    /// 按Symbol的有限档深度信息
    Depth(Depth),
}

/// 订阅WebSocket组合流时响应的数据类型
#[derive(Debug, Serialize, Deserialize)]
pub struct WsResponse {
    pub stream: String,
    pub data: Response,
}
