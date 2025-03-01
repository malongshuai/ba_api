use crate::{
    errors::{BiAnApiError, BiAnResult},
    types::order::{OrderRespType, OrderSide, OrderType, TimeInForce},
    KLineInterval, SubAccountType,
};
use ba_types::Permission;
use serde::{ser::SerializeStruct, Serialize};

/// 将Symbol列表转换为URL参数字符串
/// 例如，将["BTCUSDT", "ETHUSDT"]转换为字符串：'["BTCUSDT","ETHUSDT"]'
/// 即为每个sym元素加上双引号包围，并使用逗号连接各元素，最后将它们放进"[]"符号中间
fn list_2_str(symbols: Vec<&str>) -> Option<String> {
    if symbols.is_empty() {
        return None;
    }
    let j: Vec<String> = symbols.iter().map(|x| format!(r#""{}""#, x)).collect();
    Some(format!("[{}]", j.join(",")))
}

/// 接口鉴权类型  
/// None无需API KEY鉴权，也无需SECRET KEY签名  
/// UserStream和MarketData需API KEY鉴权，但无需SECRET KEY签名  
/// Trade和Margin和UserData需API KEY鉴权和SECRET KEY签名  
#[derive(Debug, Serialize)]
pub enum CheckType {
    /// 无需api key，也无需(sec_key)签名
    None,
    /// 需API KEY，无需(sec_key)签名
    UserStream,
    /// 需API KEY，无需(sec_key)签名
    MarketData,

    /// 需API，需(secret key)签名
    Trade,
    /// 需API，需(secret key)签名
    Margin,
    /// 需API，需(secret key)签名
    UserData,
}

pub enum PRateLimit {
    /// /api/*接口的IP限速方式
    ApiIp,
    /// /api/*接口的UID限速方式
    ApiUid,
}

/// 实现Param Trait，指定鉴权和签名类型，参考`CheckType`，
/// 参数除了实现该Trait，还需实现Serialize Trait，
/// 如果是不需要参数的请求，则定义空的Struct并实现这两个Trait即可。
pub trait Param {
    /// 是否需要鉴权和签名，默认为`CheckType::None`
    fn check_type(&self) -> CheckType {
        CheckType::None
    }

    /// 请求的限速规则
    fn rate_limit(&self) -> PRateLimit {
        PRateLimit::ApiIp
    }
}

#[derive(Debug, Serialize)]
pub struct PPing;
impl Param for PPing {}

#[derive(Debug, Serialize)]
pub struct PServerTime;
impl Param for PServerTime {}

#[derive(Debug)]
pub struct PExchangeInfo<'a> {
    // symbols: Option<Vec<&'a str>>,
    // permissions 不提供该字段时，默认包含["MARGIN", "SPOT"]，因此省略该字段，并且不能同时与symbols字段使用
    permissions: Vec<&'a str>,
    show_permission_sets: bool,
    // symbolStatus 该字段不能和symbols字段同时使用
    // symbol_status: String
}
impl<'a> PExchangeInfo<'a> {
    pub fn new(permission: Permission) -> PExchangeInfo<'a> {
        let permission_str = match permission {
            Permission::Spot => "SPOT",
            _ => {
                panic!(
                    "wrong argument `{permission:?}`, exchange_info only support SPOT Permission"
                );
            }
        };
        PExchangeInfo {
            // symbols,
            permissions: vec![permission_str],
            show_permission_sets: false,
        }
    }
}
impl<'a> Param for PExchangeInfo<'a> {}

impl<'a> Serialize for PExchangeInfo<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("PExchangeInfo", 2)?;

        // Serialize `symbols` as `["a","b","c"]` format
        // if let Some(ref symbols) = self.symbols {
        //     if !symbols.is_empty() {
        //         let ele = symbols
        //             .iter()
        //             .map(|s| format!(r#""{}""#, s))
        //             .collect::<Vec<_>>();
        //         let serialized_symbols = format!("[{}]", ele.join(","));
        //         state.serialize_field("symbols", &serialized_symbols)?;
        //     }
        // }

        // Serialize `permissions` as `["a","b","c"]` format
        let ele = self
            .permissions
            .iter()
            .map(|p| format!(r#""{}""#, p))
            .collect::<Vec<_>>();
        let serialized_permissions = format!("[{}]", ele.join(","));
        state.serialize_field("permissions", &serialized_permissions)?;

        // Serialize other fields normally
        state.serialize_field("showPermissionSets", &self.show_permission_sets)?;

        // if let Some(ref symbol_status) = self.symbol_status {
        //     state.serialize_field("SymbolStatus", symbol_status)?;
        // }

        state.end()
    }
}

#[cfg(test)]
mod test {
    use super::PExchangeInfo;
    use serde_urlencoded;
    #[test]
    fn test_p_exchange_info() {
        // test None
        let p = PExchangeInfo::new(ba_types::Permission::Spot);
        let x = serde_urlencoded::to_string(&p);
        assert_eq!(
            Ok("permissions=%5B%22SPOT%22%5D&showPermissionSets=false".to_string()),
            x,
            "None test failed"
        );

        // test empty Vec
        // p = PExchangeInfo::new(Some(vec![]));
        // x = serde_urlencoded::to_string(&p);
        // assert_eq!(
        //     Ok("showPermissionSets=false".to_string()),
        //     x,
        //     "empty Vec test failed"
        // );

        // // test Vec with one element
        // p = PExchangeInfo::new(Some(vec!["BNBBTC"]));
        // x = serde_urlencoded::to_string(&p);
        // assert_eq!(
        //     Ok("symbols=%5B%22BNBBTC%22%5D&showPermissionSets=false".to_string()),
        //     x,
        //     "one element vec test failed"
        // );

        // // test Vec with two or more element
        // p = PExchangeInfo::new(Some(vec!["BNBBTC", "BTCUSDT"]));
        // x = serde_urlencoded::to_string(&p);
        // assert_eq!(
        //     Ok("symbols=%5B%22BNBBTC%22%2C%22BTCUSDT%22%5D&showPermissionSets=false".to_string()),
        //     x,
        //     "two or more element vec test failed"
        // );
    }
}

#[derive(Debug, Serialize)]
pub struct PDepth<'a> {
    symbol: &'a str,
    limit: Option<u16>,
}
impl PDepth<'_> {
    pub fn new(symbol: &str, limit: Option<u16>) -> BiAnResult<PDepth> {
        Ok(PDepth { symbol, limit })
    }
}
impl Param for PDepth<'_> {}

#[derive(Debug, Serialize)]
pub struct PTrades<'a> {
    symbol: &'a str,
    limit: Option<u16>,
}
impl PTrades<'_> {
    pub fn new(symbol: &str, limit: Option<u16>) -> BiAnResult<PTrades> {
        // if let Some(n) = limit {
        //     if n >= 1000 {
        //         return Err(RestApiError::ArgumentError(format!(
        //             "invalid limit `{}', valid limit(<= 1000)",
        //             n
        //         )));
        //     }
        // }

        Ok(PTrades { symbol, limit })
    }
}
impl Param for PTrades<'_> {}

#[derive(Debug, Serialize)]
pub struct PHistoricalTrades<'a> {
    symbol: &'a str,
    limit: Option<u16>,
    from_id: Option<u64>,
}
impl PHistoricalTrades<'_> {
    pub fn new(
        symbol: &str,
        limit: Option<u16>,
        from_id: Option<u64>,
    ) -> BiAnResult<PHistoricalTrades> {
        // if let Some(n) = limit {
        //     if n >= 1000 {
        //         return Err(RestApiError::ArgumentError(format!(
        //             "invalid limit `{}', valid limit(<= 1000)",
        //             n
        //         )));
        //     }
        // }

        Ok(PHistoricalTrades {
            symbol,
            limit,
            from_id,
        })
    }
}
impl Param for PHistoricalTrades<'_> {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PAggTrades<'a> {
    symbol: &'a str,
    from_id: Option<u64>,
    start_time: Option<u64>,
    end_time: Option<u64>,
    limit: Option<u16>,
}
impl PAggTrades<'_> {
    pub fn new(
        symbol: &str,
        from_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> BiAnResult<PAggTrades> {
        // if let Some(n) = limit {
        //     if n >= 1000 {
        //         return Err(RestApiError::ArgumentError(format!(
        //             "invalid limit `{}', valid limit(<= 1000)",
        //             n
        //         )));
        //     }
        // }

        match (start_time, end_time) {
            (None, None) => (),
            (Some(s), Some(e)) => {
                if s >= e {
                    return Err(BiAnApiError::ArgumentError(format!(
                        "start_time({}) should small than end_time({})",
                        s, e
                    )));
                }

                if (e - s) > 3_600_000 {
                    return Err(BiAnApiError::ArgumentError(format!(
                        "end_time({}) - start_time({}) should great than 1hour",
                        e, s
                    )));
                }
            }
            _ => {
                return Err(BiAnApiError::ArgumentError(String::from(
                    "invalid start_time or end_time",
                )))
            }
        }

        Ok(PAggTrades {
            symbol,
            from_id,
            start_time,
            end_time,
            limit,
        })
    }
}
impl Param for PAggTrades<'_> {}

/// 获取K线数据的请求参数
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PKLine {
    symbol: String,
    interval: KLineInterval,
    start_time: Option<u64>,
    end_time: Option<u64>,
    limit: Option<u16>,
}

impl PKLine {
    /// 生成请求K线的请求参数
    /// ```rust
    /// let pkline = params::PKLine::new("BTCUSDT", "1m", None, None, Some(5));
    /// ```
    pub fn new(
        symbol: &str,
        interval: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> BiAnResult<Self> {
        if let (Some(s), Some(e)) = (start_time, end_time) {
            if s >= e {
                return Err(BiAnApiError::ArgumentError(format!(
                    "start_time({}) should small than end_time({})",
                    s, e
                )));
            }
        }

        Ok(Self {
            symbol: symbol.to_uppercase(),
            interval: KLineInterval::from(interval),
            start_time,
            end_time,
            limit,
        })
    }
}
impl Param for PKLine {}

#[derive(Debug, Serialize)]
pub struct PAvgPrice {
    symbol: String,
}
impl PAvgPrice {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
        }
    }
}
impl Param for PAvgPrice {}

#[derive(Debug, Serialize)]
pub struct PPrice {
    symbol: Option<String>,
    symbols: Option<String>,
}
impl PPrice {
    pub fn new(symbols: Vec<&str>) -> Self {
        if symbols.len() == 1 {
            Self {
                symbol: Some(symbols[0].into()),
                symbols: None,
            }
        } else {
            Self {
                symbols: list_2_str(symbols),
                symbol: None,
            }
        }
    }
}
impl Param for PPrice {}

#[derive(Debug, Serialize)]
pub struct PBookTicker {
    symbol: Option<String>,
    symbols: Option<String>,
}
impl PBookTicker {
    pub fn new(symbols: Vec<&str>) -> Self {
        if symbols.len() == 1 {
            Self {
                symbol: Some(symbols[0].into()),
                symbols: None,
            }
        } else {
            Self {
                symbols: list_2_str(symbols),
                symbol: None,
            }
        }
    }
}
impl Param for PBookTicker {}

#[derive(Debug, Serialize)]
pub struct PHr24 {
    // #[serde(rename = "type")]
    // tick_type: String,
    symbol: Option<String>,
    symbols: Option<String>,
}
impl PHr24 {
    pub fn new(symbols: Vec<&str>) -> Self {
        if symbols.len() == 1 {
            Self {
                symbol: Some(symbols[0].into()),
                symbols: None,
            }
        } else {
            Self {
                symbols: list_2_str(symbols),
                symbol: None,
            }
        }
    }
}
impl Param for PHr24 {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct POrder {
    symbol: String,
    side: OrderSide,
    #[serde(rename = "type")]
    order_type: OrderType,
    time_in_force: Option<TimeInForce>,
    #[serde(rename = "quantity")]
    qty: Option<String>,
    quote_order_qty: Option<String>,
    price: Option<String>,
    new_client_order_id: Option<String>,
    stop_price: Option<String>,
    iceberg_qty: Option<String>,
    new_order_resp_type: Option<OrderRespType>,
}
impl POrder {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: &str,
        side: &str,
        order_type: &str,
        time_in_force: Option<&str>,
        qty: Option<f64>,
        quote_order_qty: Option<f64>,
        price: Option<f64>,
        new_client_order_id: Option<&str>,
        stop_price: Option<f64>,
        iceberg_qty: Option<f64>,
        new_order_resp_type: Option<&str>,
    ) -> BiAnResult<POrder> {
        let side = OrderSide::from(side);
        let ot = OrderType::from(order_type);
        let tif = time_in_force.map(TimeInForce::from);
        match ot {
            OrderType::Limit => {
                if !(tif.is_some() && qty.is_some() && price.is_some()) {
                    return Err(BiAnApiError::ArgumentError(
                      format!("time_in_force({:?}), qty({:?}) and price({:?}) can't be omitted when order type is LIMIT",
                      tif, qty, price)));
                }
            }
            OrderType::Market => {
                if qty.is_none() && quote_order_qty.is_none() {
                    return Err(BiAnApiError::ArgumentError(
                      format!("qty({:?}) and quote_order_qty({:?}) can't be omitted when order type is MARKET", 
                      qty, quote_order_qty)));
                }
                if tif.is_some() {
                    return Err(BiAnApiError::ArgumentError(format!(
                        "TimeInForce({:?}) can't be set when order type is MARKET",
                        tif
                    )));
                }
            }
            OrderType::StopLoss | OrderType::TakeProfit => {
                if !(qty.is_some() && stop_price.is_some()) {
                    return Err(BiAnApiError::ArgumentError(
                      format!("qty({:?}) and stop_price({:?}) can't be omitted when order type is STOP_LOSS or TAKE_PROFIT",
                       qty, stop_price)));
                }
            }
            OrderType::StopLossLimit | OrderType::TakeProfitLimit => {
                if !(tif.is_some() && qty.is_some() && price.is_some() && stop_price.is_some()) {
                    return Err(BiAnApiError::ArgumentError(
                      format!("time_in_force({:?}), qty({:?}), price({:?}) and stop_price({:?}) can't be omitted when order type is STOP_LOSS_LIMIT or TAKE_PROFIT_LIMIT",
                       tif, qty, price, stop_price)
                    ));
                }
            }
            OrderType::LimitMaker => {
                if !(qty.is_some() && price.is_some()) {
                    return Err(BiAnApiError::ArgumentError(format!(
                        "qty({:?}) and price({:?}) can't be omitted when order type is LIMIT_MAKER",
                        qty, price
                    )));
                }
            }
        }

        Ok(POrder {
            symbol: symbol.to_uppercase(),
            side,
            order_type: ot,
            time_in_force: tif,
            qty: qty.map(|x| x.to_string()),
            quote_order_qty: quote_order_qty.map(|x| x.to_string()),
            price: price.map(|x| x.to_string()),
            new_client_order_id: new_client_order_id.map(String::from),
            stop_price: stop_price.map(|x| x.to_string()),
            iceberg_qty: iceberg_qty.map(|x| x.to_string()),
            new_order_resp_type: new_order_resp_type.map(OrderRespType::from),
        })
    }
}
impl Param for POrder {
    fn check_type(&self) -> CheckType {
        CheckType::Trade
    }

    fn rate_limit(&self) -> PRateLimit {
        PRateLimit::ApiUid
    }
}

/// 撤销订单
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PCancelOrder {
    symbol: String,
    order_id: Option<u64>,
    orig_client_order_id: Option<String>,
    new_client_order_id: Option<String>,
}

impl PCancelOrder {
    pub fn new(
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
        new_client_order_id: Option<&str>,
    ) -> BiAnResult<PCancelOrder> {
        if let (None, None) = (order_id, orig_client_order_id) {
            return Err(BiAnApiError::ArgumentError(
                "must provide one of `order_id` and `orig_client_order_id`".into(),
            ));
        }

        Ok(PCancelOrder {
            symbol: symbol.to_uppercase(),
            order_id,
            orig_client_order_id: orig_client_order_id.map(String::from),
            new_client_order_id: new_client_order_id.map(String::from),
        })
    }
}

impl Param for PCancelOrder {
    fn check_type(&self) -> CheckType {
        CheckType::Trade
    }
}

/// 撤销单一交易对的所有挂单
#[derive(Debug, Serialize)]
pub struct PCancelOpenOrders {
    symbol: String,
}
impl PCancelOpenOrders {
    pub fn new(symbol: &str) -> PCancelOpenOrders {
        PCancelOpenOrders {
            symbol: symbol.to_uppercase(),
        }
    }
}
impl Param for PCancelOpenOrders {
    fn check_type(&self) -> CheckType {
        CheckType::Trade
    }
}

/// 查询订单
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PGetOrder {
    symbol: String,
    order_id: Option<u64>,
    orig_client_order_id: Option<String>,
}
impl PGetOrder {
    pub fn new(
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
    ) -> BiAnResult<PGetOrder> {
        if let (None, None) = (order_id, orig_client_order_id) {
            return Err(BiAnApiError::ArgumentError(
                "must provide one of `order_id` and `orig_client_order_id`".into(),
            ));
        }
        Ok(PGetOrder {
            symbol: symbol.to_uppercase(),
            order_id,
            orig_client_order_id: orig_client_order_id.map(String::from),
        })
    }
}
impl Param for PGetOrder {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
}

/// 当前挂单
#[derive(Debug, Serialize)]
pub struct PGetOpenOrders {
    symbol: Option<String>,
}
impl PGetOpenOrders {
    pub fn new(symbol: Option<String>) -> PGetOpenOrders {
        PGetOpenOrders { symbol }
    }
}
impl Param for PGetOpenOrders {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PAllOrders {
    symbol: String,
    order_id: Option<u64>,
    start_time: Option<u64>,
    end_time: Option<u64>,
    limit: Option<u16>,
}
impl PAllOrders {
    pub fn new(
        symbol: &str,
        order_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> PAllOrders {
        PAllOrders {
            symbol: symbol.to_uppercase(),
            order_id,
            start_time,
            end_time,
            limit,
        }
    }
}
impl Param for PAllOrders {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
}

/// 现货交易对下架计划
#[derive(Debug, Serialize)]
pub struct PDelist;
impl Param for PDelist {
    fn check_type(&self) -> CheckType {
        CheckType::MarketData
    }
}

/// 现货交易对下架计划
#[derive(Debug, Serialize)]
pub struct PCapital;
impl Param for PCapital {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
}

/// 账户信息
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PAccount {
    omit_zero_balances: Option<bool>,
}
impl PAccount {
    pub fn new(omit_zero_balances: Option<bool>) -> Self {
        Self { omit_zero_balances }
    }
}
impl Param for PAccount {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PMyTrades {
    symbol: String,
    order_id: Option<u64>,
    start_time: Option<u64>,
    end_time: Option<u64>,
    from_id: Option<u64>,
    limit: Option<u16>,
}
impl PMyTrades {
    pub fn new(
        symbol: &str,
        order_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        from_id: Option<u64>,
        limit: Option<u16>,
    ) -> PMyTrades {
        PMyTrades {
            symbol: symbol.to_uppercase(),
            order_id,
            start_time,
            end_time,
            from_id,
            limit,
        }
    }
}
impl Param for PMyTrades {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
}

#[derive(Debug, Serialize)]
pub struct PRateLimitInfo;
impl Param for PRateLimitInfo {
    fn check_type(&self) -> CheckType {
        CheckType::Trade
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PListenKey {
    listen_key: Option<String>,
    symbol: Option<String>,
}
impl PListenKey {
    pub fn new(listen_key: Option<&str>, symbol: Option<&str>) -> Self {
        Self {
            listen_key: listen_key.map(String::from),
            symbol: symbol.map(String::from),
        }
    }
}
impl Param for PListenKey {
    fn check_type(&self) -> CheckType {
        CheckType::UserStream
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PDustBtc;
impl Param for PDustBtc {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
}

#[derive(Debug)]
pub struct PDust {
    /// 参数格式 asset=BTC&asset=USDT&asset=ETH
    asset: Vec<String>,
}

impl Serialize for PDust {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ser = serializer.serialize_struct("PDust", self.asset.len())?;
        for asset in &self.asset {
            ser.serialize_field("asset", asset)?;
        }
        ser.end()
    }
}

impl PDust {
    /// 参数格式 asset=BTC&asset=USDT&asset=ETH
    pub fn new(assets: &[&str]) -> Self {
        let mut s = Vec::new();
        for asset in assets {
            let asset = asset.to_uppercase();
            s.push(asset);
        }
        Self { asset: s }
    }
}
impl Param for PDust {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
    fn rate_limit(&self) -> PRateLimit {
        PRateLimit::ApiUid
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PSubAccountList<'a> {
    email: Option<&'a str>,
    is_freeze: Option<&'a str>,
}

impl<'a> PSubAccountList<'a> {
    pub fn new(email: Option<&'a str>, is_freeze: Option<&'a str>) -> Self {
        Self { email, is_freeze }
    }
}
impl Param for PSubAccountList<'_> {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
    fn rate_limit(&self) -> PRateLimit {
        PRateLimit::ApiIp
    }
}

#[derive(Debug, Serialize)]
pub struct PSubAccountAssets<'a> {
    email: &'a str,
}

impl<'a> PSubAccountAssets<'a> {
    pub fn new(email: &'a str) -> Self {
        Self { email }
    }
}
impl Param for PSubAccountAssets<'_> {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
    fn rate_limit(&self) -> PRateLimit {
        PRateLimit::ApiUid
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PSubAccountUniversalTransfer<'a> {
    from_email: Option<&'a str>,
    to_email: Option<&'a str>,
    from_account_type: SubAccountType,
    to_account_type: SubAccountType,
    symbol: Option<&'a str>,
    asset: String,
    amount: f64,
}

impl<'a> PSubAccountUniversalTransfer<'a> {
    pub fn new(
        from_email: Option<&'a str>,
        to_email: Option<&'a str>,
        from_account_type: &str,
        to_account_type: &str,
        asset: &'a str,
        amount: f64,
        symbol: Option<&'a str>,
    ) -> BiAnResult<Self> {
        if from_email.is_none() && to_email.is_none() {
            return Err(BiAnApiError::ArgumentError(
                "both of from_email and to_email are missing".into(),
            ));
        }
        Ok(Self {
            from_email,
            to_email,
            from_account_type: SubAccountType::from(from_account_type),
            to_account_type: SubAccountType::from(to_account_type),
            symbol,
            asset: asset.to_uppercase(),
            amount,
        })
    }
}
impl Param for PSubAccountUniversalTransfer<'_> {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
    fn rate_limit(&self) -> PRateLimit {
        PRateLimit::ApiUid
    }
}

#[derive(Debug, Serialize)]
pub struct PAccountInfo;
impl Param for PAccountInfo {
    fn check_type(&self) -> CheckType {
        CheckType::UserData
    }
    fn rate_limit(&self) -> PRateLimit {
        PRateLimit::ApiUid
    }
}
