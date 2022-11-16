use crate::client::string_to_f64;
use serde::{Deserialize, Serialize};

/// 某币对btc和对bnb的等值关系
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DustBtcInfo {
    /// 资产名
    pub asset: String,
    /// 资产全名
    pub asset_full_name: String,
    /// 可转换数量
    #[serde(deserialize_with = "string_to_f64")]
    pub amount_free: f64,
    /// 等值btc
    #[serde(rename = "toBTC", deserialize_with = "string_to_f64")]
    pub to_btc: f64,
    /// 可转换BNB（未扣除手续费）
    #[serde(rename = "toBNB", deserialize_with = "string_to_f64")]
    pub to_bnb: f64,
    /// 可转换BNB（已扣除手续费）
    #[serde(rename = "toBNBOffExchange", deserialize_with = "string_to_f64")]
    pub to_bnb_off_exchange: f64,
    /// 手续费
    #[serde(deserialize_with = "string_to_f64")]
    pub exchange: f64,
}

/// 所有小额资产对btc和对bnb的等值关系
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DustBtc {
    /// 全部资产等值BTC
    #[serde(deserialize_with = "string_to_f64")]
    pub total_transfer_btc: f64,
    /// 总共可以转换的BNB数量
    #[serde(rename = "totalTransferBNB", deserialize_with = "string_to_f64")]
    pub total_transfer_bnb: f64,
    /// 转换手续费
    #[serde(rename = "dribbletPercentage", deserialize_with = "string_to_f64")]
    pub transfer_fee: f64,
    /// 列表信息
    pub details: Vec<DustBtcInfo>,
}

/// 小额资产转换为bnb的转换结果信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferInfo {
    #[serde(deserialize_with = "string_to_f64")]
    pub amount: f64,
    pub from_asset: String,
    pub operate_time: u64,
    pub tran_id: u64,
    #[serde(deserialize_with = "string_to_f64")]
    pub service_charge_amount: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub transfered_amount: f64,
}

/// 小额资产转换为bnb的转换结果信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dust {
    #[serde(deserialize_with = "string_to_f64")]
    pub total_service_charge: f64,
    #[serde(deserialize_with = "string_to_f64")]
    pub total_transfered: f64,
    pub transfer_result: Vec<TransferInfo>,
}

#[cfg(test)]
mod wallet_test {
    use crate::types::wallet::{Dust, DustBtc};

    #[test]
    fn test_dust_btc() {
        let str = r##"
            {
                "details": [
                    {
                        "asset": "ADA",
                        "assetFullName": "ADA",
                        "amountFree": "6.21",
                        "toBTC": "0.00016848", 
                        "toBNB": "0.01777302", 
                        "toBNBOffExchange": "0.01741756", 
                        "exchange": "0.00035546" 
                    }
                ],
                "totalTransferBtc": "0.00016848",
                "totalTransferBNB": "0.01777302",
                "dribbletPercentage": "0.02"
            }
        "##;
        println!("{:?}", serde_json::from_str::<DustBtc>(str));
    }

    #[test]
    fn test_dust() {
        let str = r##"
            {
                "totalServiceCharge":"0.02102542",
                "totalTransfered":"1.05127099",
                "transferResult":[
                    {
                        "amount":"0.03000000",
                        "fromAsset":"ETH",
                        "operateTime":1563368549307,
                        "serviceChargeAmount":"0.00500000",
                        "tranId":2970932918,
                        "transferedAmount":"0.25000000"
                    },
                    {
                        "amount":"0.09000000",
                        "fromAsset":"LTC",
                        "operateTime":1563368549404,
                        "serviceChargeAmount":"0.01548000",
                        "tranId":2970932918,
                        "transferedAmount":"0.77400000"
                    },
                    {
                        "amount":"248.61878453",
                        "fromAsset":"TRX",
                        "operateTime":1563368549489,
                        "serviceChargeAmount":"0.00054542",
                        "tranId":2970932918,
                        "transferedAmount":"0.02727099"
                    }
                ]
            }
        "##;
        println!("{:?}", serde_json::from_str::<Dust>(str));
    }
}
