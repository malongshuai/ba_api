use tracing::instrument;

use super::{
    params::{PSubAccountAssets, PSubAccountList, PSubAccountUniversalTransfer},
    RestConn,
};
use crate::{
    errors::BiAnResult,
    types::sub_account::{SubAccountBalances, SubAccounts, UniversalTransfer},
    Balances,
};

/// 字母账户相关接口
impl RestConn {
    /// 可进行小额资产转换的币种和数量
    #[instrument(skip(self))]
    pub async fn sub_account_list(
        &self,
        email: Option<&str>,
        is_freeze: Option<&str>,
    ) -> BiAnResult<SubAccounts> {
        let path = "/sapi/v1/sub-account/list";
        let params = PSubAccountList::new(email, is_freeze);
        let res = self.rest_req("get", path, params, Some(1)).await?;
        let sub_account_list = serde_json::from_str::<SubAccounts>(&res)?;
        Ok(sub_account_list)
    }

    /// 查询子账户资产
    #[instrument(skip(self))]
    pub async fn sub_account_assests(&self, email: &str) -> BiAnResult<Balances> {
        let path = "/sapi/v3/sub-account/assets";
        let params = PSubAccountAssets::new(email);
        let res = self.rest_req("get", path, params, Some(1)).await?;
        let sub_account_balances = serde_json::from_str::<SubAccountBalances>(&res)?;
        Ok(sub_account_balances.balances)
    }

    /// 子母账户资产万能划转：母账户 <=> 子账户 <=> 子账户
    /// 只能母账户调用，且母账户API开启了子母划转权限
    /// from_email和to_email至少给一个：
    ///  - 不给from_email时，表示从母账户划转到指定的子账户
    ///  - 不给to_email时，表示从指定的子账户划转到母账户
    /// from_account_type和to_account_type可接受的值(不区分大小写)：
    ///  - spot
    ///  - usdtfuture/usdt_future
    ///  - coinfuture/coin_future
    ///  - isolatedmargin/isolated_margin
    /// symbol参数只能结合isolatedmargin、isolated_margin一起使用
    #[instrument(skip(self))]
    #[allow(clippy::too_many_arguments)]
    pub async fn sub_account_univ_trans(
        &self,
        from_email: Option<&str>,
        to_email: Option<&str>,
        from_account_type: &str,
        to_account_type: &str,
        asset: &str,
        amount: f64,
        symbol: Option<&str>,
    ) -> BiAnResult<UniversalTransfer> {
        let path = "/sapi/v1/sub-account/universalTransfer";
        let params = PSubAccountUniversalTransfer::new(
            from_email,
            to_email,
            from_account_type,
            to_account_type,
            asset,
            amount,
            symbol,
        )?;
        let res = self.rest_req("post", path, params, Some(1)).await?;
        tracing::info!("{}", res);
        let res = serde_json::from_str::<UniversalTransfer>(&res)?;
        Ok(res)
    }

    /// 子母账户现货资产划转：母账户 <=> 子账户 <=> 子账户
    /// 只能母账户调用，且母账户API开启了子母划转权限
    /// from_email和to_email至少给一个：
    ///  - 不给from_email时，表示从母账户划转到指定的子账户
    ///  - 不给to_email时，表示从指定的子账户划转到母账户
    #[instrument(skip(self))]
    pub async fn sub_account_spot_trans(
        &self,
        from_email: Option<&str>,
        to_email: Option<&str>,
        asset: &str,
        amount: f64,
    ) -> BiAnResult<UniversalTransfer> {
        self.sub_account_univ_trans(from_email, to_email, "spot", "spot", asset, amount, None)
            .await
    }
}
