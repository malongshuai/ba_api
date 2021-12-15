use super::{
    params::{
        PAccount, PAllOrders, PCancelOpenOrders, PCancelOrder, PGetOpenOrders, PGetOrder,
        PListenKey, PMyTrades, POrder, PRateLimitInfo, Param,
    },
    RestConn,
};
use crate::{
    errors::BiAnResult,
    types::{
        account::{Account, ListenKey},
        order::{CancelOpenOrdersInfo, CancelOrderInfo, MyTrades, Order, OrderInfo},
        rate_limit::RateLimitInfo,
    },
};
use serde::Serialize;
use std::fmt::Debug;
use tracing::instrument;

/// 现货账户和现货交易接口
impl RestConn {
    /// 获取现货账户信息
    #[instrument(skip(self))]
    pub async fn account(&self) -> BiAnResult<Account> {
        let path = "/api/v3/account";
        let params = PAccount;
        let res = self.rest_req("get", path, params).await?;
        let account_info = serde_json::from_str::<Account>(&res)?;
        Ok(account_info)
    }

    /// 现货下单接口
    /// side: 不区分大小的 buy/sell
    /// order_type: 订单类型，值为以下几种不区分大小写的值，不同类型的订单，强制要求提供的参数不同
    ///   - Limit(限价单)
    ///   - Market(市价单)
    ///   - StopLoss(止损单)
    ///   - StopLossLimit(限价止损单)
    ///   - TakeProfit(止盈单)
    ///   - TakeProfitLimit(限价止盈单)
    ///   - LimitMaker(限价只挂单)
    ///
    /// time_in_force: 订单有效方式，不区分大小写的 gtc/ioc/fok
    /// qty: 币的数量
    /// quote_order_qty：市价单中，报价资产的数量。例如买入BTCUSDT时，表示买入多少USDT的BTC
    /// stop_price: 止盈止损单的止盈止损价
    /// iceberg_qty: Limit和LimitMaker单时指定该参数，表示变成冰山单，此时time_in_force必须为GTC类型
    /// new_order_resp_type: 指定下单后的响应信息的详细程度，值为不区分大小写的 ack/result/full
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self))]
    pub async fn order(
        &self,
        symbol: &str,
        side: &str,
        order_type: &str,
        time_in_force: Option<&str>,
        qty: Option<f64>,
        quote_order_qty: Option<f64>,
        price: Option<f64>,
        new_client_order_id: Option<String>,
        stop_price: Option<f64>,
        iceberg_qty: Option<f64>,
        new_order_resp_type: Option<&str>,
    ) -> BiAnResult<Order> {
        let path = "/api/v3/order";
        let params = POrder::new(
            symbol,
            side,
            order_type,
            time_in_force,
            qty,
            quote_order_qty,
            price,
            new_client_order_id,
            stop_price,
            iceberg_qty,
            new_order_resp_type,
        )?;
        let res = self.rest_req("post", path, params).await?;
        let order_info = serde_json::from_str::<Order>(&res)?;
        Ok(order_info)
    }

    /// 限价单接口
    /// side: 不区分大小写的 buy/sell
    /// price: 买入或卖出的挂单价格
    /// qty:
    ///   - 当side为买入时，表示买入报价资产的数量，将自动根据price参数的值转换为币的数量。例如要买入BTCUSDT，该数量表示要买入多少USDT的BTC
    ///   - 当side为卖出时，表示要卖出的币的数量
    #[instrument(skip(self))]
    pub async fn limit_order(
        &self,
        symbol: &str,
        side: &str,
        qty: f64,
        price: f64,
    ) -> BiAnResult<Order> {
        let amount = if side.to_lowercase() == "sell" {
            qty
        } else {
            // 转换为币的数量
            qty / price
        };
        let order = self
            .order(
                symbol,
                side,
                "limit",
                Some("gtc"),
                Some(amount),
                None,
                Some(price),
                None,
                None,
                None,
                Some("Full"),
            )
            .await?;
        Ok(order)
    }

    /// 撤单
    /// order_id和orig_client_order_id必须指定一个，指定前者表示根据order_id进行撤单，指定后者表示根据订单的client_order_id进行撤单。new_client_order_id是为当前撤单操作指定一个client_order_id，若省略则自动生成
    #[instrument(skip(self))]
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
        new_client_order_id: Option<&str>,
    ) -> BiAnResult<CancelOrderInfo> {
        let path = "/api/v3/order";
        let params =
            PCancelOrder::new(symbol, order_id, orig_client_order_id, new_client_order_id)?;
        let res = self.rest_req("delete", path, params).await?;
        let cancel = serde_json::from_str::<CancelOrderInfo>(&res)?;
        Ok(cancel)
    }

    /// 撤单(撤销某个交易对下的所有挂单，包括OCO挂单)
    #[instrument(skip(self))]
    pub async fn cancel_open_orders(&self, symbol: &str) -> BiAnResult<Vec<CancelOpenOrdersInfo>> {
        let path = "/api/v3/openOrders";
        let params = PCancelOpenOrders::new(symbol)?;
        let res = self.rest_req("delete", path, params).await?;
        let cancel = serde_json::from_str::<Vec<CancelOpenOrdersInfo>>(&res)?;
        Ok(cancel)
    }

    /// 查询订单信息
    #[instrument(skip(self))]
    pub async fn get_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
    ) -> BiAnResult<OrderInfo> {
        let path = "/api/v3/order";
        let params = PGetOrder::new(symbol, order_id, orig_client_order_id)?;
        let res = self.rest_req("get", path, params).await?;
        let order_info = serde_json::from_str::<OrderInfo>(&res)?;
        Ok(order_info)
    }

    /// 查询某交易对或所有交易对下的所有当前挂单信息
    #[instrument(skip(self))]
    pub async fn get_open_orders(&self, symbol: &str) -> BiAnResult<Vec<OrderInfo>> {
        let path = "/api/v3/openOrders";
        let params = PGetOpenOrders::new(symbol)?;
        let res = self.rest_req("get", path, params).await?;
        let open_orders_info = serde_json::from_str::<Vec<OrderInfo>>(&res)?;
        Ok(open_orders_info)
    }

    /// 查询某交易对或所有交易对下的所有当前挂单信息
    #[instrument(skip(self))]
    pub async fn get_all_orders(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> BiAnResult<Vec<OrderInfo>> {
        let path = "/api/v3/allOrders";
        let params = PAllOrders::new(symbol, order_id, start_time, end_time, limit)?;
        let res = self.rest_req("get", path, params).await?;
        let all_orders_info = serde_json::from_str::<Vec<OrderInfo>>(&res)?;
        Ok(all_orders_info)
    }

    /// 获取账户指定交易对的成交历史
    #[instrument(skip(self))]
    pub async fn my_trades(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        from_id: Option<u64>,
        limit: Option<u16>,
    ) -> BiAnResult<Vec<MyTrades>> {
        let path = "/api/v3/myTrades";
        let params = PMyTrades::new(symbol, order_id, start_time, end_time, from_id, limit)?;
        let res = self.rest_req("get", path, params).await?;
        let trades_info = serde_json::from_str::<Vec<MyTrades>>(&res)?;
        Ok(trades_info)
    }

    /// 查询目前下单数
    #[instrument(skip(self))]
    pub async fn rate_limit_info(&self) -> BiAnResult<Vec<RateLimitInfo>> {
        let path = "/api/v3/rateLimit/order";
        let params = PRateLimitInfo;
        let res = self.rest_req("get", path, params).await?;
        let rate_limit_info = serde_json::from_str::<Vec<RateLimitInfo>>(&res)?;
        Ok(rate_limit_info)
    }
}

/// Listen Key接口
impl RestConn {
    #[instrument(skip(self))]
    async fn listen_key<P: Serialize + Param + Debug>(
        &self,
        account_type: &str,
        action: &str,
        params: P,
    ) -> BiAnResult<String> {
        let path = match account_type {
            "spot" => "/api/v3/userDataStream",
            "margin" => "/sapi/v1/userDataStream",
            "margin_isolated" => "/sapi/v1/userDataStream/isolated",
            _ => panic!("error account, valid type: spot/margin/margin_isolated"),
        };

        let method = match action {
            "create" => "post",
            "delay" => "put",
            "delete" => "delete",
            _ => panic!("error action, valid action: post/put/delete"),
        };

        let res = self.rest_req(method, path, params).await?;
        let listen_key = serde_json::from_str::<ListenKey>(&res)?;
        Ok(listen_key.listen_key)
    }

    /// 生成或延迟现货账户的ListenKey，如果当前现货账户没有ListenKey，则生成一个新的ListenKey，有效期60分钟，如果已有，则延长该ListenKey有效期60分钟
    #[instrument(skip(self))]
    pub async fn new_spot_listen_key(&self) -> BiAnResult<String> {
        let params = PListenKey::new(None, None);
        let listen_key = self.listen_key("spot", "create", params).await?;
        Ok(listen_key)
    }

    /// 延迟现货账户的ListenKey，有效期延长至本次调用后60分钟
    #[instrument(skip(self))]
    pub async fn delay_spot_listen_key(&self, listen_key: &str) -> BiAnResult<()> {
        let params = PListenKey::new(Some(listen_key), None);
        self.listen_key("spot", "delay", params).await?;
        Ok(())
    }

    /// 删除现货账户的ListenKey，将关闭用户数据流
    #[instrument(skip(self))]
    pub async fn delete_spot_listen_key(&self, listen_key: &str) -> BiAnResult<()> {
        let params = PListenKey::new(Some(listen_key), None);
        self.listen_key("spot", "delete", params).await?;

        Ok(())
    }

    /// 生成或延迟杠杆账户的ListenKey，如果当前账户没有ListenKey，则生成一个新的ListenKey，有效期60分钟，如果已有，则延长该ListenKey有效期60分钟
    #[instrument(skip(self))]
    pub async fn new_margin_listen_key(&self) -> BiAnResult<String> {
        let params = PListenKey::new(None, None);
        let listen_key = self.listen_key("margin", "create", params).await?;
        Ok(listen_key)
    }

    /// 延迟杠杆账户的ListenKey，有效期延长至本次调用后60分钟
    #[instrument(skip(self))]
    pub async fn delay_margin_listen_key(&self, listen_key: &str) -> BiAnResult<()> {
        let params = PListenKey::new(Some(listen_key), None);
        self.listen_key("margin", "delay", params).await?;
        Ok(())
    }

    /// 删除杠杆账户的ListenKey，将关闭用户数据流
    #[instrument(skip(self))]
    pub async fn delete_margin_listen_key(&self, listen_key: &str) -> BiAnResult<()> {
        let params = PListenKey::new(Some(listen_key), None);
        self.listen_key("margin", "delete", params).await?;

        Ok(())
    }

    /// 生成或延迟逐仓杠杆账户的ListenKey，如果当前账户没有ListenKey，则生成一个新的ListenKey，有效期60分钟，如果已有，则延长该ListenKey有效期60分钟
    #[instrument(skip(self))]
    pub async fn new_isolated_margin_listen_key(&self, symbol: &str) -> BiAnResult<String> {
        let params = PListenKey::new(None, Some(symbol));
        let listen_key = self.listen_key("margin_isolated", "create", params).await?;
        Ok(listen_key)
    }

    /// 延迟逐仓杠杆账户的ListenKey，有效期延长至本次调用后60分钟
    #[instrument(skip(self))]
    pub async fn delay_isolated_margin_listen_key(
        &self,
        listen_key: &str,
        symbol: &str,
    ) -> BiAnResult<()> {
        let params = PListenKey::new(Some(listen_key), Some(symbol));
        self.listen_key("margin_isolated", "delay", params).await?;
        Ok(())
    }

    /// 删除逐仓杠杆账户的ListenKey，将关闭用户数据流
    #[instrument(skip(self))]
    pub async fn delete_isolated_margin_listen_key(
        &self,
        listen_key: &str,
        symbol: &str,
    ) -> BiAnResult<()> {
        let params = PListenKey::new(Some(listen_key), Some(symbol));
        self.listen_key("margin_isolated", "delete", params).await?;

        Ok(())
    }
}
