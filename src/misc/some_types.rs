use ba_types::string_to_f64;
use serde::{Deserialize, Serialize};
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

/// 如果确实要下架，下架信息存放在data字段(OfflineMessage类型)中
#[derive(Debug, Deserialize)]
pub(super) struct OfflineCheck {
    pub(super) code: String,
    pub(super) success: bool,
    pub(super) message: Option<String>,
    pub(super) messageDetail: Option<String>,
    pub(super) data: Option<OfflineMessage>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct OfflineMessage {
    /// 是否将要下架
    pub(super) pom: bool,
    /// 将要下架的时间(毫秒Epoch)
    pub(super) pomt: Option<u64>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CryptoInfoCheck {
    pub(super) code: String,
    pub(super) success: bool,
    pub(super) message: Option<String>,
    pub(super) messageDetail: Option<String>,
    pub(super) data: CryptoInfo,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CryptoInfo {
    /// 别名（例如：ETH），注意，不带USDT等后缀
    #[serde(alias = "alias")]
    pub coin_name: String,

    /// 市值
    #[serde(alias = "mc")]
    pub market_cap: f64,

    /// 排名
    #[serde(alias = "rk")]
    pub rank: u32,

    /// 市场占有率
    #[serde(alias = "dmc")]
    pub dominance: Option<f64>,

    /// 流通数量
    #[serde(alias = "cs")]
    pub circulating_supply: u64,

    // /// 原始名称（例如：Ethereum, 1000*SATS）
    // #[serde(alias = "sb")]
    // pub name: String,
    /// 最大供应量（该资产生命周期中将存在的最大代币数量，可能为空）
    ///
    /// 如果不为空值，跟 total_supply 都表示总供应量，但两者的值有可能是不一样的,,
    /// 如果和total_supply的值不一样，通常max_supply会更大一些，
    /// 因为total_supply是被燃烧一部分代币后的总量
    #[serde(alias = "ms")]
    pub max_supply: Option<u64>,

    /// 总供应量（已发行总量减去已销毁部分）
    #[serde(alias = "ts")]
    pub total_supply: u64,

    /// 完全稀释市值
    #[serde(alias = "fdmc")]
    pub fully_diluted_market_cap: f64,
    /// 图标地址
    #[serde(alias = "url")]
    icon_url: String,
    /// 发行日期（时间戳，毫秒）
    #[serde(alias = "id")]
    pub issue_date: Option<u64>,
    /// 成交量
    #[serde(alias = "v", deserialize_with = "string_to_f64")]
    pub volume: f64,
    /// 成交量 / 市值（百分比）
    #[serde(alias = "vpm", deserialize_with = "string_to_f64")]
    pub volume_to_market_cap: f64,
    /// 发行价
    #[serde(alias = "ipu")]
    pub issue_price: Option<f64>,
    /// 外部链接（用逗号分隔）
    #[serde(alias = "eu")]
    pub external_urls: Option<String>,
    /// 网站链接
    #[serde(alias = "ws")]
    pub website: Option<String>,
    /// 历史最高价
    #[serde(alias = "athpu")]
    pub all_time_high_price: f64,
    /// 达到最高价的时间（时间戳，毫秒）
    #[serde(alias = "athd")]
    pub all_time_high_date: u64,
    /// 是否已达到历史最高价
    #[serde(alias = "athfc")]
    pub all_time_high_flag: bool,
    /// 最低价
    #[serde(alias = "atlpu")]
    pub all_time_low_price: Option<f64>,
    /// 达到最低价的时间（时间戳，毫秒）
    #[serde(alias = "ald")]
    pub all_time_low_date: u64,
    /// 是否已达到历史最低价
    #[serde(alias = "atlfc")]
    pub all_time_low_flag: bool,
    /// 白皮书链接
    #[serde(alias = "wpu")]
    pub white_paper_url: String,
    /// 其他描述信息
    #[serde(alias = "dbk")]
    pub description_key: String,
}

#[cfg(test)]
mod tt {
    use super::CryptoInfo;

    #[test]
    fn serde_crypto_info() {
        let str = r#"
        {
            "sb": "Ethereum",
            "alias": "ETH",
            "url": "https://static-file-1306379396.file.myqcloud.com/image/admin_mgs_image_upload/20201110/3a8c9fe6-2a76-4ace-aa07-415d994de6f0.png",
            "mc": 433207567914.24384000,
            "rk": 2,
            "cs": 120436471,
            "v": "43162437095.26909000",
            "vpm": "0.09963454",
            "ms": null,
            "ts": 120436471,
            "id": 1406160000000,
            "ipu": 0.308000000000000000000000,
            "eu": "https://etherscan.io/,https://solscan.io/token/2FPyTwcZLUg1MDrwsyoP4D6s1tM7hAkHYRjkNb5w6Pxk,https://bscscan.com/token/0x2170ed0880ac9a755fd29b2688956bd959f933f8,https://www.okx.com/web3/explorer/eth,https://blockchair.com/ethereum",
            "ws": "https://www.ethereum.org/",
            "dmc": 13.18,
            "athpu": 4891.704697551414000000000000000000,
            "athd": 1637047500000,
            "athfc": true,
            "atlpu": 0.420897006988525400000000000000,
            "ald": 1445467200000,
            "atlfc": true,
            "fdmc": 433207567914.24000000,
            "wpu": "https://github.com/ethereum/wiki/wiki/White-Paper",
            "dbk": "symbol_desc_ETH"
        }
    "#;

        let ci1 = serde_json::from_str::<CryptoInfo>(str).unwrap();
        println!("{:?}", ci1);
        let new_json_str = serde_json::to_string(&ci1).unwrap();
        println!("{}", new_json_str);
        let ci2 = serde_json::from_str::<CryptoInfo>(&new_json_str).unwrap();
        println!("{:?}", ci2);
    }

    #[test]
    fn serde_crypto_info1() {
        let str = r#"
{"sb":"Bitcoin","alias":"BTC","url":"https://static-file-1306379396.file.myqcloud.com/image/admin_mgs_image_upload/20201110/87496d50-2408-43e1-ad4c-78b47b448a6a.png","mc":1903935750689.64900000,"rk":1,"cs":19788550,"v":"70578226016.39973000","vpm":"0.03706964","ms":21000000,"ts":19788550,"id":1225497600000,"ipu":null,"eu":"https://blockchain.info/,https://live.blockcypher.com/btc/,https://blockchair.com/bitcoin,https://explorer.viabtc.com/btc,https://www.okx.com/web3/explorer/btc","ws":"https://bitcoin.org/","dmc":57.48,"athpu":99655.501078633530000000000000000000,"athd":1732304400000,"athfc":true,"atlpu":0.048646540000000000000000000000,"ald":1279135500000,"atlfc":true,"fdmc":2020494213294.18000000,"wpu":"https://bitcoin.org/bitcoin.pdf","dbk":"symbol_desc_BTC"}
    "#;

        let ci1 = serde_json::from_str::<CryptoInfo>(str).unwrap();
        println!("{:?}", ci1);
        let new_json_str = serde_json::to_string(&ci1).unwrap();
        println!("{}", new_json_str);
        let ci2 = serde_json::from_str::<CryptoInfo>(&new_json_str).unwrap();
        println!("{:?}", ci2);
    }
}
