#![allow(dead_code, non_snake_case)]

use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::warn;

use crate::errors::BiAnResult;

/// 如果确实有警告，警告内容存放在data字段(WarnMessage类型)的content字段中
#[derive(Debug, Deserialize)]
struct WarnInfo {
    code: String,
    success: bool,
    message: Option<String>,
    messageDetail: Option<String>,
    data: Option<WarnMessage>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct WarnMessage {
    content: Option<String>,
    title: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

/// 查看该币当前是否存在警告信息
/// 有时候某些币因为存在基本面的风险(如团队消失、申请破产、波动太大等等)，Bian会给出警告信息。
/// 这些信息目前无法从API获取，因此只能从网页给出的地址来爬
pub async fn check_coin_warning(sym: &str) -> BiAnResult<Option<String>> {
    let url = "https://www.binance.com/bapi/capital/v1/friendly/marketing/symbolDisclaimer/querySymbolDisclaimer";
    let post_body = json!({
      "tradingPairs": sym,
      "type": "spot"
    });

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .json(&post_body)
        .header("lang", "zh-CN")
        .send()
        .await?
        .json::<WarnInfo>()
        .await?;

    if res.data.is_some() {
        Ok(res.data.unwrap().content)
    } else if !res.success {
        warn!("check coin({}) warn failed: {:?}", sym, res.message);
        Ok(None)
    } else {
        Ok(None)
    }
}
