#![allow(dead_code, non_snake_case)]

use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, warn};

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

    info!("check coin warning: {:?}", res);

    if res.data.is_some() {
        Ok(res.data.unwrap().content)
    } else if !res.success {
        warn!("check coin({}) warn failed: {:?}", sym, res.message);
        Ok(None)
    } else {
        Ok(None)
    }
}

/// 如果确实要下架，下架信息存放在data字段(OfflineMessage类型)中
#[derive(Debug, Deserialize)]
struct OfflineInfo {
    code: String,
    success: bool,
    message: Option<String>,
    messageDetail: Option<String>,
    data: Option<OfflineMessage>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct OfflineMessage {
    /// 是否将要下架
    pom: bool,
    /// 将要下架的时间(毫秒Epoch)
    pomt: Option<u64>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

/// 检查该交易对是否将要下架(根据警告信息检查).
/// sym忽略大小写，例如:
///
/// ```text
/// check_offline("btcusdt")
/// check_offline("Btcbusd")
/// ```
///
/// 如果确实要下架，返回Ok(Some(Epoch))，Epoch是毫秒级，表示下架时间点。
/// 返回Err表示请求错误，返回Ok(None)表示不下架或请求的交易对名称不对
///
/// 也可以通过rest_conn的`delist_schedule()`方法来判断
pub async fn check_offline(sym: &str) -> BiAnResult<Option<u64>> {
    let sym = sym.to_ascii_uppercase();

    let mut url = String::from(
        "https://www.binance.com/bapi/asset/v2/public/asset-service/product/get-product-by-symbol",
    );
    url.reserve(18);
    url.push_str("?symbol=");
    url.push_str(&sym);

    // let url = format!("{}?symbol={}", url, sym);

    let client = reqwest::Client::new();
    let res = client.get(url).send().await?.json::<OfflineInfo>().await?;
    info!("check offline: {:?}", res);

    if res.data.is_some() {
        if let Some(off_msg) = res.data {
            if off_msg.pom {
                return Ok(off_msg.pomt);
            }
        }
    }
    Ok(None)
}
