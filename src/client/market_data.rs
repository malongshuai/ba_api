use std::time::SystemTime;

use tracing::instrument;

use crate::errors::RestResult;
use crate::types::depth::Depth;
use crate::types::order::{AggTrade, HistoricalTrade, Trade};
use crate::types::other_types::{AvgPrice, Prices, ServerTime};
use crate::types::symbol_info::ExchangeInfo;
use crate::types::ticker::{BookTickers, FullTickers};
use crate::KLines;

use super::params::{
    PAggTrades, PAvgPrice, PBookTicker, PDepth, PExchangeInfo, PHistoricalTrades, PHr24, PKLine,
    PPing, PPrice, PServerTime, PTrades,
};
use super::RestConn;

/// 行情接口
impl RestConn {
    /// 测试连通性，连通时返回true
    #[instrument(skip(self))]
    pub async fn ping(&self) -> RestResult<bool> {
        let path = "/api/v3/ping";
        let res = self.rest_req("get", path, PPing).await?;
        Ok(res == "{}")
    }

    /// 获取服务器时间，获取成功时返回u64
    #[instrument(skip(self))]
    pub async fn server_time(&self) -> RestResult<u64> {
        let path = "/api/v3/time";
        let res = self.rest_req("get", path, PServerTime).await?;
        let time_res = serde_json::from_str::<ServerTime>(&res)?;
        Ok(time_res.server_time)
    }

    /// 获取交易对信息
    /// ```rust
    /// // 获取所有交易对信息
    /// rest_conn.exchange_info(None);
    /// rest_conn.exchange_info(Some(vec![]));
    ///
    /// // 获取一个或多个交易堆信息
    /// rest_conn.exchange_info(Some(vec!["BTCUSDT"]));
    /// ```
    #[instrument(skip(self))]
    pub async fn exchange_info(&self, symbols: Option<Vec<&str>>) -> RestResult<ExchangeInfo> {
        let path = "/api/v3/exchangeInfo";
        let params = PExchangeInfo::new(symbols);
        let res = self.rest_req("get", path, params).await?;
        let exchange_info = serde_json::from_str::<ExchangeInfo>(&res)?;
        Ok(exchange_info)
    }

    /// 获取指定币的深度信息(limit为None时默认返回买盘和卖盘各100条信息)
    #[instrument(skip(self))]
    pub async fn depth(&self, symbol: &str, limit: Option<u16>) -> RestResult<Depth> {
        let path = "/api/v3/depth";
        let params = PDepth::new(symbol, limit)?;
        let res = self.rest_req("get", path, params).await?;
        let depth = serde_json::from_str::<Depth>(&res)?;
        Ok(depth)
    }

    /// 近期成交列表(limit为None时默认返回最近500条信息)
    #[instrument(skip(self))]
    pub async fn trades(&self, symbol: &str, limit: Option<u16>) -> RestResult<Vec<Trade>> {
        let path = "/api/v3/trades";
        let params = PTrades::new(symbol, limit)?;
        let res = self.rest_req("get", path, params).await?;
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
    ) -> RestResult<Vec<HistoricalTrade>> {
        let path = "/api/v3/historicalTrades";
        let params = PHistoricalTrades::new(symbol, limit, from_id)?;
        let res = self.rest_req("get", path, params).await?;
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
    ) -> RestResult<Vec<AggTrade>> {
        let path = "/api/v3/aggTrades";
        let params = PAggTrades::new(symbol, from_id, start_time, end_time, limit)?;
        let res = self.rest_req("get", path, params).await?;
        let agg_trades = serde_json::from_str::<Vec<AggTrade>>(&res)?;
        Ok(agg_trades)
    }

    /// 获取K线列表  
    /// 
    /// interval: 1m 3m 5m 15m 30m 1h 2h 4h 6h 8h 12h 1d 3d 1w 1M，  
    /// limit为None时默认返回最近500条信息，最大值1000，  
    /// start_time太小时，自动调整为币安的第一根K线时间，  
    /// end_time太大时，最多返回到当前的K线结束，  
    /// 注：如果获取的是最近的K线，对于最后一根K线是否finish，有最多3秒的延迟期。建议丢掉最近的最后一根K线，或者总是将其当作未完成的K线来看待。  
    /// 例如，最后一根K线是某分钟00秒 01秒 02秒时获取到的，则也认为这根K线是未完成的  
    #[instrument(skip(self))]
    pub async fn klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> RestResult<KLines> {
        let path = "/api/v3/klines";
        let params = PKLine::new(symbol, interval, start_time, end_time, limit)?;
        let res = self.rest_req("get", path, params).await?;
        let mut klines = serde_json::from_str::<KLines>(&res)?;

        for kl in &mut klines {
          kl.symbol = symbol.to_string();
        }

        // 如果最后一根K线的close_epoch大于当前时间(延迟3秒)，则认为这根K线未完成
        if !klines.is_empty() {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let last_close_epoch = klines.last().unwrap().close_epoch;
            if now < (last_close_epoch as u128) + 3000 {
                klines.last_mut().unwrap().finished = false;
            }
        }

        Ok(klines)
    }

    /// 获取当前均价(币安提供当前5分钟的均价，5分钟内的总成交额除以总成交量)  
    #[instrument(skip(self))]
    pub async fn avg_price(&self, symbol: &str) -> RestResult<AvgPrice> {
        let path = "/api/v3/avgPrice";
        let params = PAvgPrice::new(symbol);
        let res = self.rest_req("get", path, params).await?;
        let avg_price = serde_json::from_str::<AvgPrice>(&res)?;
        Ok(avg_price)
    }

    /// 获取某交易对或所有交易对的最新价格(实时价)  
    /// symbol为None时返回所有交易对的实时价格
    #[instrument(skip(self))]
    pub async fn price(&self, symbol: Option<&str>) -> RestResult<Prices> {
        let path = "/api/v3/ticker/price";
        let params = PPrice::new(symbol);
        let res = self.rest_req("get", path, params).await?;
        let prices = serde_json::from_str::<Prices>(&res)?;
        Ok(prices)
    }

    /// 获取某交易对或所有交易对的最优挂单价  
    /// symbol为None时返回所有交易对的信息
    #[instrument(skip(self))]
    pub async fn book_ticker(&self, symbol: Option<&str>) -> RestResult<BookTickers> {
        let path = "/api/v3/ticker/bookTicker";
        let params = PBookTicker::new(symbol);
        let res = self.rest_req("get", path, params).await?;
        let tickers = serde_json::from_str::<BookTickers>(&res)?;
        Ok(tickers)
    }

    /// 获取某交易对或所有交易对的24小时价格变动信息  
    /// symbol为None时返回所有交易对的24时价格变动信息(返回数据量巨大，且请求的权重极大)
    #[instrument(skip(self))]
    pub async fn hr24(&self, symbol: Option<&str>) -> RestResult<FullTickers> {
        let path = "/api/v3/ticker/24hr";
        let params = PHr24::new(symbol);
        let res = self.rest_req("get", path, params).await?;
        let hrs = serde_json::from_str::<FullTickers>(&res)?;
        Ok(hrs)
    }
}
