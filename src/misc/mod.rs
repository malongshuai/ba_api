#![allow(dead_code, non_snake_case)]

pub mod some_types;

use crate::errors::BiAnResult;
use serde_json::json;
use some_types::{TradingPairInfoWrap, WarnInfo};
use tracing::{info, warn};

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

    // 下面几个URL，都可以一次性获取所有交易对的信息，
    // 其中包含每个交易对是否下架以及如果下架其毫秒级的下架Epoch，但需要修改它的时区
    // - https://www.binance.com/bapi/asset/v2/public/asset-service/product/get-products
    // - https://www.binance.com/bapi/margin/v1/friendly/margin/symbols
    // - https://www.binance.com/bapi/margin/v1/public/isolated-margin/pair/listed
    let mut url = String::from(
        "https://www.binance.com/bapi/asset/v2/public/asset-service/product/get-product-by-symbol",
    );
    url.reserve(18);
    url.push_str("?symbol=");
    url.push_str(&sym);

    // let url = format!("{}?symbol={}", url, sym);

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .send()
        .await?
        .json::<TradingPairInfoWrap>()
        .await?;
    info!("check offline: {:?}", res);

    if res.data.is_some() {
        if let Some(off_msg) = res.data {
            if off_msg.to_delist {
                return Ok(off_msg.delist_time);
            }
        }
    }
    Ok(None)
}
