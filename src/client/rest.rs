use crate::{
    errors::{BadRequest, MethodError, RestApiError, RestResult},
    REST_BASE_URL,
};
use reqwest::{header, Url};
use serde::Serialize;
use std::{fmt::Debug, str::FromStr};
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

/// 已建立好的Http连接客户端(reqwest::Client)
#[derive(Debug, Clone)]
pub struct RestConn {
    conn: reqwest::Client,
    api_key: String,
    sec_key: String,
    base_url: Url,
}

#[allow(dead_code)]
impl RestConn {
    ///```rust
    ///let api_key = "abcdefhijklmnopqrstuvwxyz".to_string();
    ///let sec_key = "abcdefhijklmnopqrstuvwxyz".to_string();
    ///let rest_conn = RestConn::new(api_key, sec_key, Some("http://127.0.0.1:8118".to_string()));
    ///```
    pub fn new(api_key: String, sec_key: String, proxy: Option<String>) -> RestConn {
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
        let url = *REST_BASE_URL;

        match proxy {
            Some(prx) => {
                let p = reqwest::Proxy::all(prx).expect("proxy error!");
                Self {
                    conn: builder.proxy(p).build().unwrap(),
                    api_key,
                    sec_key,
                    base_url: Url::parse(url).unwrap(),
                }
            }
            None => Self {
                conn: builder.build().unwrap(),
                api_key,
                sec_key,
                base_url: Url::parse(url).unwrap(),
            },
        }
    }

    async fn check_rest_resp(resp: reqwest::Response) -> RestResult<reqwest::Response> {
        let status_code = u16::from(resp.status());
        if status_code >= 300 {
            let e = if status_code >= 500 {
                RestApiError::ServerError(resp.text().await.unwrap_or_default())
            } else if status_code == 400 {
                let resp_test = resp.text().await.unwrap_or_default();
                match serde_json::from_str::<BadRequest>(resp_test.as_str()) {
                    Ok(error) => RestApiError::BadRequest(error.code, error.msg),
                    Err(_) => RestApiError::ClientError(resp_test),
                }
            } else if status_code == 403 {
                RestApiError::Waf
            } else if status_code == 418 {
                RestApiError::Blocked
            } else if status_code == 429 {
                RestApiError::WafWarning
            } else if status_code > 400 {
                RestApiError::ClientError(resp.text().await.unwrap_or_default())
            } else {
                RestApiError::Unknown(resp.text().await.unwrap_or_default())
            };
            Err(e)
        } else {
            Ok(resp)
        }
    }

    fn make_url<P>(&self, path: &str, params: P) -> Url
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
    ///let body = rest_conn.rest_req("get", pkline_path, pkline).await.unwrap();
    ///```
    #[tracing::instrument(skip(self))]
    pub async fn rest_req<P>(&self, method: &str, path: &str, params: P) -> RestResult<RespBody>
    where
        P: Serialize + Param + Debug,
    {
        let url = self.make_url(path, params);

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

        // 尝试重连3次的方案
        let mut retry = 4;
        let mut resp = loop {
            retry -= 1;
            if retry != 3 {
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
                        error!("connect failed {}", e.url().unwrap().to_string());
                        if retry == 0 {
                            break Err(RestApiError::ConnectError(e.to_string()));
                        }
                        continue;
                    }
                    break Err(RestApiError::RequestError(e));
                }
            };
        }?;

        resp = Self::check_rest_resp(resp).await?;
        Ok(resp.text().await.unwrap())
    }
}
