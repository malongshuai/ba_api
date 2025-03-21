use ba_global::app_dir;
use ba_types::Permission;

use super::{
    params::{PCapital, PDelist},
    rate_limit::RateLimitParam,
};
use {
    super::{
        params::{
            PAggTrades, PAvgPrice, PBookTicker, PDepth, PExchangeInfo, PHistoricalTrades, PHr24,
            PKLine, PPing, PPrice, PServerTime, PTrades,
        },
        RestConn,
    },
    crate::errors::BiAnResult,
    crate::types::depth::Depth,
    crate::types::order::{AggTrade, HistoricalTrade, Trade},
    crate::types::other_types::{AvgPrice, Prices, ServerTime},
    crate::types::symbol_info::{DelistSchedule, ExchangeInfo},
    crate::types::ticker::{BookTickers, FullTickers},
    crate::{KLineInterval, KLines},
    std::{error, path::Path, time::SystemTime},
    tokio::{fs, io::AsyncReadExt},
    tracing::instrument,
};

/// 行情接口
impl RestConn {
    /// 测试连通性，连通时返回true
    #[instrument(skip(self))]
    pub async fn ping(&self) -> BiAnResult<bool> {
        let path = "/api/v3/ping";
        let res = self
            .rest_req("get", path, PPing, RateLimitParam::Weight(1))
            .await?;
        Ok(res == "{}")
    }

    /// 获取服务器时间，获取成功时返回u64
    #[instrument(skip(self))]
    pub async fn server_time(&self) -> BiAnResult<u64> {
        let path = "/api/v3/time";
        let res = self
            .rest_req("get", path, PServerTime, RateLimitParam::Weight(1))
            .await?;
        let time_res = serde_json::from_str::<ServerTime>(&res)?;
        Ok(time_res.server_time)
    }

    /// 尝试从给定文件中读取exchange_info信息，如果能读取，且该文件的mtime在半小时以内，则返回Ok(Some(ExchangeInfo))，其它返回值情况都表示读取失败或本地数据无效
    async fn local_exchange_info(
        exchange_info_file: &Path,
    ) -> Result<Option<ExchangeInfo>, Box<dyn error::Error>> {
        let mut file = fs::File::open(&exchange_info_file).await?;

        let mtime = file.metadata().await?.modified()?;
        let duration = SystemTime::now().duration_since(mtime)?.as_secs();
        if duration > 1800 {
            return Ok(None);
        }

        let mut buf = String::new();
        file.read_to_string(&mut buf).await?;
        Ok(Some(serde_json::from_str::<ExchangeInfo>(&buf)?))
    }

    /// 获取交易对信息
    /// ```rust
    /// // 获取所有现货交易对的信息
    /// rest_conn.exchange_info();
    /// ```
    ///
    /// 注意，为了减小响应体积，关闭了响应中的权限字段permissionSets字段的显示，
    /// 但该方法默认获取的均为Spot现货交易对，并且手动添加到permissionSets字段，
    /// 如果该现货交易对同时支持Margin杠杆交易，则也会将该权限手动追加到permissionSets字段，
    ///
    /// 因此，该方法获取所有现货交易对，其中一部分可能还同时支持Margin交易
    #[instrument(skip(self))]
    pub async fn exchange_info(&self) -> BiAnResult<ExchangeInfo> {
        let path = "/api/v3/exchangeInfo";
        let permission = Permission::Spot;
        let params = PExchangeInfo::new(permission);

        // 如果本地文件已有exchange_info的信息，且文件的mtime在半小时以内，则从本地文件读取数据并返回，否则请求新数据并写入本地文件
        let bian_dir = app_dir().unwrap().join("bian");
        let exchange_info_file = bian_dir.join("exchange_info.json");
        let c_res = fs::create_dir_all(&bian_dir).await;
        if c_res.is_ok() {
            if let Ok(Some(exchange_info)) = Self::local_exchange_info(&exchange_info_file).await {
                return Ok(exchange_info);
            }
        }

        let res = self
            .rest_req("get", path, params, RateLimitParam::Weight(20))
            .await?;

        let mut exchange_info = serde_json::from_str::<ExchangeInfo>(&res)?;
        // 由于默认关闭了响应中的PermissionSets的显示，
        // 因此此处手动将permission全部填充到各个交易对信息中
        let mut p: Vec<Permission> = Vec::with_capacity(2);
        p.push(permission);
        for si in exchange_info.symbols.iter_mut() {
            let mut pp = p.clone();
            if si.is_margin_trading_allowed {
                pp.push(Permission::Margin);
            }
            si.permission_sets.push(pp);
        }

        // 移除非USDT交易对的信息，减小体积
        exchange_info.symbols.retain(|si| si.quote_asset == "USDT");

        // 保存到本地
        if c_res.is_ok() {
            match serde_json::to_string(&exchange_info) {
                Ok(data) => {
                    let _ = fs::write(&exchange_info_file, data.as_bytes()).await;
                }
                Err(e) => {
                    tracing::error!("can't serialize exchange_info: {e}");
                }
            }
        }

        Ok(exchange_info)
    }

    /// 获取指定币的深度信息(limit为None时默认返回买盘和卖盘各100条信息)
    #[instrument(skip(self))]
    pub async fn depth(&self, symbol: &str, limit: Option<u16>) -> BiAnResult<Depth> {
        let path = "/api/v3/depth";

        let rate_limit = match limit {
            Some(n) => match n {
                1..=100 => 5,
                101..=500 => 25,
                501..=1000 => 50,
                1001..=5000 => 250,
                _ => 250,
            },
            None => 2,
        };

        let params = PDepth::new(symbol, limit)?;
        let res = self
            .rest_req("get", path, params, RateLimitParam::Weight(rate_limit))
            .await?;
        let depth = serde_json::from_str::<Depth>(&res)?;
        Ok(depth)
    }

    /// 近期成交列表(limit为None时默认返回最近500条信息)
    #[instrument(skip(self))]
    pub async fn trades(&self, symbol: &str, limit: Option<u16>) -> BiAnResult<Vec<Trade>> {
        let path = "/api/v3/trades";
        let params = PTrades::new(symbol, limit)?;
        let res = self
            .rest_req("get", path, params, RateLimitParam::Weight(25))
            .await?;
        let trades = serde_json::from_str::<Vec<Trade>>(&res)?;
        Ok(trades)
    }

    /// 查询历史成交列表(limit为None时默认返回最近500条信息，from_id为None时默认返回最近信息)
    #[instrument(skip(self))]
    pub async fn historical_trades(
        &self,
        symbol: &str,
        limit: Option<u16>,
        from_id: Option<u64>,
    ) -> BiAnResult<Vec<HistoricalTrade>> {
        let path = "/api/v3/historicalTrades";
        let params = PHistoricalTrades::new(symbol, limit, from_id)?;
        let res = self
            .rest_req("get", path, params, RateLimitParam::Weight(25))
            .await?;
        let historical_trades = serde_json::from_str::<Vec<HistoricalTrade>>(&res)?;
        Ok(historical_trades)
    }

    /// 查询归集成交列表  
    /// limit为None时默认返回最近500条信息，from_id为None时默认返回最近信息，
    /// start_time和end_time需同时提供或同时不提供，同时提供时，两者间隔需小于一小时
    #[instrument(skip(self))]
    pub async fn agg_trades(
        &self,
        symbol: &str,
        from_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> BiAnResult<Vec<AggTrade>> {
        let path = "/api/v3/aggTrades";
        let params = PAggTrades::new(symbol, from_id, start_time, end_time, limit)?;
        let res = self
            .rest_req("get", path, params, RateLimitParam::Weight(4))
            .await?;
        let agg_trades = serde_json::from_str::<Vec<AggTrade>>(&res)?;
        Ok(agg_trades)
    }

    /// 获取K线列表  
    ///
    /// interval: 1m 3m 5m 15m 30m 1h 2h 4h 6h 8h 12h 1d 3d 1w 1M，  
    /// limit为None时默认返回最近500条信息，最大值1000，  
    /// start_time太小时，自动调整为币安的第一根K线时间，  
    /// end_time太大时，最多返回到当前的K线结束，  
    /// 注：如果获取的是最近的K线，
    /// 1.如果最后一根K线的close_epoch大于请求前的时间点，且大于请求后时间点超过2秒，则认为这根K线是未完成的.
    /// 2.有些K线交易量较小，可能获取到的最近的K线中，最后一根K线是几分钟前的。币安在K线未更新时不会产生新K线。
    #[instrument(skip(self))]
    pub async fn klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> BiAnResult<KLines> {
        let path = "/api/v3/klines";
        let params = PKLine::new(symbol, interval, start_time, end_time, limit)?;
        let now_bf = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let res = self
            .rest_req("get", path, params, RateLimitParam::Weight(2))
            .await?;
        let now_af = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let mut klines = serde_json::from_str::<KLines>(&res)?;

        for kl in &mut klines {
            kl.symbol = symbol.into();
            kl.interval = KLineInterval::from(interval);
        }

        // 如果最后一根K线的close_epoch大于请求前的时间点，且大于请求后时间点超过2秒，则认为这根K线是未完成的
        if !klines.is_empty() {
            let last_close_epoch = klines.last().unwrap().close_epoch as u128;
            if last_close_epoch > now_bf && (last_close_epoch - now_af) > 2000 {
                klines.last_mut().unwrap().finish = false;
            }
        }

        Ok(klines)
    }

    /// 获取当前均价(币安提供当前5分钟的均价，5分钟内的总成交额除以总成交量)  
    #[instrument(skip(self))]
    pub async fn avg_price(&self, symbol: &str) -> BiAnResult<AvgPrice> {
        let path = "/api/v3/avgPrice";
        let params = PAvgPrice::new(symbol);
        let res = self
            .rest_req("get", path, params, RateLimitParam::Weight(2))
            .await?;
        let avg_price = serde_json::from_str::<AvgPrice>(&res)?;
        Ok(avg_price)
    }

    /// 获取某交易对或所有交易对的24小时价格变动的详细信息  
    /// symbols为空时返回所有交易对的24时价格变动信息(返回数据量巨大，且请求的权重极大)
    #[instrument(skip(self))]
    pub async fn hr24(&self, symbols: Vec<&str>) -> BiAnResult<FullTickers> {
        let path = "/api/v3/ticker/24hr";
        let rate_limit = match &symbols.len() {
            1..=20 => 2,
            21..=100 => 40,
            _ => 80,
        };
        let params = PHr24::new(symbols);
        let res = self
            .rest_req("get", path, params, RateLimitParam::Weight(rate_limit))
            .await?;
        let hrs = serde_json::from_str::<FullTickers>(&res)?;
        Ok(hrs)
    }

    /// 获取某交易对或所有交易对的最新价格(实时价)  
    /// symbol为空时返回所有交易对的实时价格
    #[instrument(skip(self))]
    pub async fn price(&self, symbols: Vec<&str>) -> BiAnResult<Prices> {
        let path = "/api/v3/ticker/price";
        let rate_limit = match symbols.len() {
            1 => 2,
            _ => 4,
        };
        let params = PPrice::new(symbols);
        let res = self
            .rest_req("get", path, params, RateLimitParam::Weight(rate_limit))
            .await?;
        let prices = serde_json::from_str::<Prices>(&res)?;
        Ok(prices)
    }

    /// 获取某交易对或所有交易对的最优挂单价  
    /// symbol为空时返回所有交易对的信息
    #[instrument(skip(self))]
    pub async fn book_ticker(&self, symbols: Vec<&str>) -> BiAnResult<BookTickers> {
        let path = "/api/v3/ticker/bookTicker";
        let rate_limit = match symbols.len() {
            1 => 2,
            _ => 4,
        };
        let params = PBookTicker::new(symbols);
        let res = self
            .rest_req("get", path, params, RateLimitParam::Weight(rate_limit))
            .await?;
        let tickers = serde_json::from_str::<BookTickers>(&res)?;
        Ok(tickers)
    }

    /// 查询现货下架计划(下架交易对列表)
    #[instrument(skip(self))]
    pub async fn delist_schedule(&self) -> BiAnResult<DelistSchedule> {
        let path = "/sapi/v1/spot/delist-schedule";
        let res = self
            .rest_req("get", path, PDelist, RateLimitParam::Weight(100))
            .await?;
        let infos = serde_json::from_str::<DelistSchedule>(&res)?;
        Ok(infos)
    }

    /// 针对账户的所有资产信息
    #[instrument(skip(self))]
    pub async fn capital(&self) -> BiAnResult<String> {
        let path = "/sapi/v1/capital/config/getall";
        let res = self
            .rest_req("get", path, PCapital, RateLimitParam::Weight(10))
            .await?;
        Ok(res)
        // let infos = serde_json::from_str::<DelistSchedule>(&res)?;
        // Ok(infos)
    }
}
