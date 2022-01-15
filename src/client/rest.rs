use crate::{
    api_rate_limit::APIRateLimit,
    client::params::PRateLimit,
    errors::{BadRequest, BiAnApiError, BiAnResult, MethodError},
    utils::ExchangeInfoExt,
    ExchangeInfo, SymbolInfo, REST_BASE_URL,
};
use reqwest::{header, Url};
use serde::Serialize;
use std::{fmt::Debug, str::FromStr, sync::Arc, time::Duration};
use tokio::time;
use tracing::{error, warn};

use super::params::{CheckType, Param};

/// REST响应体
pub(crate) type RespBody = String;

mod helper {
    use ring::hmac;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn timestamp() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    pub fn signature(key: &str, obj: &str) -> String {
        let key_bytes = key.as_bytes();
        let obj_bytes = obj.as_bytes();

        let sign_key = hmac::Key::new(hmac::HMAC_SHA256, key_bytes);
        let sign = hmac::sign(&sign_key, obj_bytes);
        hex::encode(sign)
    }

    #[test]
    fn test_signature() {
        let key = "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j";
        let obj = "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559";
        assert_eq!(
            signature(key, obj),
            "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71".to_string()
        )
    }
}

/// 币安只支持GET POST PUT和DELETE四种HTTP方法
/// ```rust
/// assert_eq!(RestMethod::from_str('get'), RestMethod::Get);
/// assert_eq!(RestMethod::from_str('GET'), RestMethod::Get);
/// ```
#[derive(Debug, PartialEq)]
pub enum RestMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl FromStr for RestMethod {
    type Err = MethodError;

    ///```rust
    /// RestMethod::from("get");
    /// RestMethod::from("Get");
    /// RestMethod::from("GET");
    ///```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let m = s.to_lowercase();
        match m.as_str() {
            "get" => Ok(RestMethod::Get),
            "put" => Ok(RestMethod::Put),
            "post" => Ok(RestMethod::Post),
            "delete" => Ok(RestMethod::Delete),
            _ => Err(MethodError { msg: m }),
        }
    }
}

#[allow(dead_code)]
/// 已建立好的Http连接客户端(reqwest::Client)
#[derive(Debug, Clone)]
pub struct RestConn {
    conn: reqwest::Client,
    api_key: String,
    sec_key: String,
    base_url: Url,
    rate_limit: APIRateLimit,
    exchange_info: Arc<Option<ExchangeInfo>>,
}

#[allow(dead_code)]
impl RestConn {
    ///```rust
    ///let api_key = "abcdefhijklmnopqrstuvwxyz".to_string();
    ///let sec_key = "abcdefhijklmnopqrstuvwxyz".to_string();
    ///let rest_conn = RestConn::new(api_key, sec_key, Some("http://127.0.0.1:8118".to_string()));
    ///```
    pub async fn new(api_key: String, sec_key: String, proxy: Option<String>) -> RestConn {
        let mut header = header::HeaderMap::new();
        header.insert(
            "X-MBX-APIKEY",
            header::HeaderValue::from_str(&api_key).unwrap(),
        );

        // 设置建立连接过程的超时时间5秒
        // 空闲连接永不断开(一直保持长连接)
        let builder = reqwest::Client::builder()
            .default_headers(header)
            .connect_timeout(time::Duration::from_secs(5))
            .pool_idle_timeout(None);
        // let url = *REST_BASE_URL;

        let conn = match proxy {
            Some(prx) => {
                let p = reqwest::Proxy::all(prx).expect("proxy error!");
                builder.proxy(p).build().unwrap()
            }
            None => builder.build().unwrap(),
        };

        let mut rest_conn = RestConn {
            conn,
            api_key,
            sec_key,
            base_url: Url::parse(*REST_BASE_URL).unwrap(),
            rate_limit: APIRateLimit::new().await,
            exchange_info: Arc::new(None),
        };

        if let Ok(exchange_info) = rest_conn.exchange_info(None).await {
            rest_conn.exchange_info = Arc::new(Some(exchange_info));
        };

        rest_conn
    }

    /// 更新exchange_info信息
    pub async fn update_exchange_info(&mut self) -> BiAnResult<()> {
        match self.exchange_info(None).await {
            Ok(exchange_info) => {
                self.exchange_info = Arc::new(Some(exchange_info));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// 获取交易对信息
    pub fn get_exchange_info(&self) -> Option<&ExchangeInfo> {
        self.exchange_info.as_ref().as_ref()
    }

    /// 获取交易对的信息，以便能够调整价格、数量
    pub fn symbol_info(&self, symbol: &str) -> Option<SymbolInfo> {
        match self.get_exchange_info() {
            Some(ex_info) => ex_info.symbol_info(symbol).cloned(),
            None => None,
        }
    }

    async fn check_rest_resp(resp: reqwest::Response) -> BiAnResult<reqwest::Response> {
        let status_code = u16::from(resp.status());
        if status_code >= 300 {
            let e = if status_code >= 500 {
                BiAnApiError::ServerError(resp.text().await.unwrap_or_default())
            } else if status_code == 400 {
                let resp_test = resp.text().await.unwrap_or_default();
                match serde_json::from_str::<BadRequest>(resp_test.as_str()) {
                    Ok(error) => BiAnApiError::BadRequest(error.code, error.msg),
                    Err(_) => BiAnApiError::ClientError(resp_test),
                }
            } else if status_code == 403 {
                BiAnApiError::Waf
            } else if status_code == 418 {
                BiAnApiError::Blocked
            } else if status_code == 429 {
                BiAnApiError::WafWarning
            } else if status_code > 400 {
                BiAnApiError::ClientError(resp.text().await.unwrap_or_default())
            } else {
                BiAnApiError::Unknown(resp.text().await.unwrap_or_default())
            };
            Err(e)
        } else {
            Ok(resp)
        }
    }

    /// 生成完整的URL
    fn make_url<P>(&self, path: &str, params: &P) -> Url
    where
        P: Serialize + Param + Debug,
    {
        let mut url = self.base_url.join(path).expect("invalid url");

        match params.check_type() {
            CheckType::None | CheckType::UserStream | CheckType::MarketData => {
                let query = serde_urlencoded::to_string(params).unwrap();
                if !query.is_empty() {
                    url.set_query(Some(&query));
                }
            }
            CheckType::Trade | CheckType::Margin | CheckType::UserData => {
                let mut query = serde_urlencoded::to_string(params).unwrap();
                let time_query = format!("recvWindow=5000&timestamp={}", helper::timestamp());
                query = if query.is_empty() {
                    time_query
                } else {
                    format!("{}&{}", query, time_query)
                };

                let signature = helper::signature(&self.sec_key, &query);
                query = format!("{}&signature={}", query, signature);
                url.set_query(Some(&query));
            }
        };
        url
    }

    /// REST请求，返回响应的Body字符串，否则报错
    ///```rust
    ///use ba_api::KLineInterval;
    ///use ba_api::client::params;
    ///use ba_api::client::RestConn;
    ///
    ///let api_key = "TAfI8PqyqOYiegSoijSy";
    ///let sec_key = "MOIeI2mK13IEyrLHfCwh";
    ///let rest_conn = RestConn::new(api_key.to_string(), sec_key.to_string(), None);
    ///
    ///let pkline_path = "/api/v3/klines";
    ///let pkline = params::PKLine {
    ///  symbol: "BTCUSDT".to_string(),
    ///  interval: KLineInterval::from("1m"),
    ///  start_time: None,
    ///  end_time: None,
    ///  limit: Some(5u16),
    ///};
    ///
    ///let body = rest_conn.rest_req("get", pkline_path, pkline, Some(1)).await.unwrap();
    ///```
    #[tracing::instrument(skip(self))]
    pub async fn rest_req<P>(
        &self,
        method: &str,
        path: &str,
        params: P,
        rate_limit: Option<usize>,
    ) -> BiAnResult<RespBody>
    where
        P: Serialize + Param + Debug,
    {
        let url = self.make_url(path, &params);

        // 获取限速值
        if let Some(n) = rate_limit {
            match params.rate_limit() {
                PRateLimit::ApiIp => {
                    self.rate_limit.get_ip_permit(n).await;
                }
                PRateLimit::ApiUid => {
                    self.rate_limit.get_uid_permit(n).await;
                }
            }
        }

        // 不重连的方案
        // let mut resp = match RestMethod::from_str(method)? {
        //     RestMethod::Get => self.conn.get(url),
        //     RestMethod::Post => self.conn.post(url),
        //     RestMethod::Put => self.conn.put(url),
        //     RestMethod::Delete => self.conn.delete(url),
        // }
        // .send()
        // .await?;
        // resp = Self::check_rest_resp(resp).await?;
        // Ok(resp.text().await.unwrap())

        // 尝试重连5次的方案(每隔1秒重试一次)
        let mut retry = 6;
        let mut resp = loop {
            retry -= 1;
            if retry != 5 {
                warn!("Connect retry, remain retry times: {}", retry);
            }

            let url = url.clone();
            let req = match RestMethod::from_str(method)? {
                RestMethod::Get => self.conn.get(url),
                RestMethod::Post => self.conn.post(url),
                RestMethod::Put => self.conn.put(url),
                RestMethod::Delete => self.conn.delete(url),
            };

            match req.send().await {
                Ok(r) => break Ok(r),
                Err(e) => {
                    if e.is_connect() || e.is_timeout() {
                        error!("connect failed<{}>: {}", e.url().unwrap().to_string(), e);
                        if retry == 0 {
                            break Err(BiAnApiError::ConnectError(e.to_string()));
                        }
                        // 每次重试，隔一秒
                        time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                    break Err(BiAnApiError::RequestError(e));
                }
            };
        }?;

        let head = resp.headers();
        // println!("header: {:?}", head);
        // println!( "+ weight +{:?} {:?}", head.get("x-mbx-used-weight"), head.get("x-mbx-used-weight-1m") );
        // println!( "+ weight +{:?} {:?}", head.get("x-mbx-order-count-10s"), head.get("x-mbx-order-count-1d") );

        // 将返回的已用权重值设置到当前的剩余权重中
        self.set_rate_limit(head).await;

        resp = Self::check_rest_resp(resp).await?;
        Ok(resp.text().await.unwrap())
    }

    async fn set_rate_limit(&self, head: &header::HeaderMap) {
        // 此为`/api/*`的IP限速
        if let Some(used) = self.extract_weigth_header("x-mbx-used-weight-1m", head) {
            self.rate_limit.set_ip_permit(used).await;
        }
        // 此为`/api/*`的UID限速
        if let (Some(sec_used), Some(day_used)) = (
            self.extract_weigth_header("x-mbx-order-count-10s", head),
            self.extract_weigth_header("x-mbx-order-count-1d", head),
        ) {
            self.rate_limit.set_uid_permit(sec_used, day_used).await;
        }
    }

    fn extract_weigth_header(&self, key: &str, head: &header::HeaderMap) -> Option<usize> {
        if let Some(weight) = head.get(key) {
            if let Ok(used) = weight.to_str() {
                return used.parse::<usize>().ok();
            }
        }
        None
    }

    /// 返回此刻剩余的限速权重,第一个是ip权重,第二个值为uid的秒级权重,第三个值为日级权重
    pub async fn rate_limit_remains(&self) -> (usize, usize, usize) {
        let ip_rate_limit = self.rate_limit.ip_permits_remain().await;
        let (sec_uid, day_uid) = self.rate_limit.uid_permits_remain().await;
        (ip_rate_limit, sec_uid, day_uid)
    }
}
