#![allow(dead_code, non_snake_case)]

pub mod some_types;

use crate::errors::BiAnResult;
use serde_json::json;
use some_types::{CryptoInfoCheck, OfflineCheck, WarnInfo};
use tracing::{info, warn};

pub use some_types::CryptoInfo;

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

    let mut url = String::from(
        "https://www.binance.com/bapi/asset/v2/public/asset-service/product/get-product-by-symbol",
    );
    url.reserve(18);
    url.push_str("?symbol=");
    url.push_str(&sym);

    // let url = format!("{}?symbol={}", url, sym);

    let client = reqwest::Client::new();
    let res = client.get(url).send().await?.json::<OfflineCheck>().await?;
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

/// 获取币种的基本信息，包括排名、市值、市占率、流通量、总供应量
///
/// sym忽略大小写，可省略后缀，例如:
///
/// ```text
/// get_crypto_info("btcusdt")
/// get_crypto_info("ETHusdt")
/// get_crypto_info("ETH")
/// get_crypto_info("eth")
/// ```
pub async fn get_crypto_info(sym: &str) -> BiAnResult<CryptoInfo> {
    // pub async fn get_crypto_info(sym: &str) -> BiAnResult<()> {
    // 如果含有USDT后缀，去掉
    let mut sym = sym.to_ascii_uppercase();
    if sym.ends_with("USDT") {
        sym = sym.strip_suffix("USDT").unwrap().to_string();
    }

    // https://www.binance.com/bapi/apex/v1/friendly/apex/marketing/tardingPair/detail?symbol=GFT
    let mut url = String::from(
        "https://www.binance.com/bapi/apex/v1/friendly/apex/marketing/tardingPair/detail",
    );
    url.reserve(18);
    url.push_str("?symbol=");
    url.push_str(&sym);

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .send()
        .await?
        // .json::<CryptoInfoCheck>()
        .text()
        .await?;
    let res = match serde_json::from_str::<CryptoInfoCheck>(&res) {
        Ok(res) => res,
        Err(_) => {
            println!("{res}");
            return Err(crate::errors::BiAnApiError::Unknown(res));
        }
    };
    // println!("{res}");
    // Ok(())
    Ok(res.data)
}
