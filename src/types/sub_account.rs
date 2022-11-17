use serde::Deserialize;

use crate::Balances;

/// 子账户列表
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubAccounts {
    sub_accounts: Vec<SubAccountInfo>,
}

impl SubAccounts {
    /// 所有子账户
    pub fn accounts(self) -> Vec<SubAccountInfo> {
        self.sub_accounts
    }
}

/// 子账户信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubAccountInfo {
    pub email: String,
    pub is_freeze: bool,
    pub create_time: u64,
    pub is_managed_sub_account: bool,
    pub is_asset_management_sub_account: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubAccountBalances {
    pub(crate) balances: Balances,
    #[allow(dead_code)]
    pub(crate) success: bool,
}

/// 字母账户万能划转的响应
/// 子账户信息
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniversalTransfer {
    pub tran_id: u64,
}
