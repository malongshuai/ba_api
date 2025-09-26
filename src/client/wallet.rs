use super::{
    params::{PDust, PDustBtc},
    rate_limit::RateLimitParam,
    RestConn,
};
use crate::{
    errors::BiAnResult,
    types::wallet::{Dust, DustBtc},
};
use tracing::instrument;

/// 钱包相关接口
impl RestConn {
    /// 可进行小额资产转换的币种和数量
    #[instrument(skip(self))]
    pub async fn dust_list(&self) -> BiAnResult<DustBtc> {
        let path = "/sapi/v1/asset/dust-btc";
        let res = self
            .rest_req("post", path, PDustBtc::new(), RateLimitParam::Weight(1))
            .await?;
        let dust_list_info = serde_json::from_str::<DustBtc>(&res)?;
        Ok(dust_list_info)
    }

    /// 小额资产转换，每6小时转换一次
    #[instrument(skip(self))]
    pub async fn dust(&self, assets: &[&str]) -> BiAnResult<Dust> {
        let path = "/sapi/v1/asset/dust";
        let params = PDust::new(assets);
        let res = self
            .rest_req("post", path, params, RateLimitParam::Weight(10))
            .await?;
        let dust_info = serde_json::from_str::<Dust>(&res)?;
        Ok(dust_info)
    }
}
