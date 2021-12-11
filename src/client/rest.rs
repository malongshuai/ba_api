use crate::{
    client::helper,
    errors::{MethodError, RestApiError, RestResult},
};
use reqwest::{header, Url};
use serde::Serialize;
use std::{fmt::Debug, str::FromStr};

use super::params::{CheckType, Param};

/// 币安的Base URL: <https://api.binance.com>
pub const BASE_URL: &str = "https://api.binance.com";

/// REST响应体
pub(crate) type RespBody = String;

/// 币安只支持GET POST PUT和DELETE四种HTTP方法
/// ```rust
/// use ba_api::client::RestMethod;
///
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
    /// use ba_api::client::RestMethod;
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
#[derive(Debug)]
pub struct RestConn {
    conn: reqwest::Client,
    api_key: String,
    sec_key: String,
    base_url: Url,
}

#[allow(dead_code)]
impl RestConn {
    ///```rust
    ///use ba_api::client::RestConn;
    ///
    ///let api_key = "abcdefhijklmnopqrstuvwxyz";
    ///let sec_key = "abcdefhijklmnopqrstuvwxyz";
    ///let rest_conn = RestConn::new(api_key, sec_key, Some("http://127.0.0.1:8118"));
    ///```
    pub fn new(api_key: &'static str, sec_key: &'static str, proxy: Option<&str>) -> RestConn {
        let mut header = header::HeaderMap::new();
        header.insert("X-MBX-APIKEY", header::HeaderValue::from_static(api_key));

        let builder = reqwest::Client::builder().default_headers(header);

        match proxy {
            Some(prx) => {
                let p = reqwest::Proxy::all(prx).expect("proxy error!");
                Self {
                    conn: builder.proxy(p).build().unwrap(),
                    api_key: api_key.to_string(),
                    sec_key: sec_key.to_string(),
                    base_url: Url::parse(BASE_URL).unwrap(),
                }
            }
            None => Self {
                conn: builder.build().unwrap(),
                api_key: api_key.to_string(),
                sec_key: sec_key.to_string(),
                base_url: Url::parse(BASE_URL).unwrap(),
            },
        }
    }

    async fn check_rest_resp(resp: reqwest::Response) -> RestResult<reqwest::Response> {
        let status_code = u16::from(resp.status());
        if status_code >= 300 {
            let e = if status_code >= 500 {
                RestApiError::ServerError(resp.text().await.unwrap_or_default())
            } else if status_code == 418 {
                RestApiError::Blocked
            } else if status_code == 429 {
                RestApiError::WafWarning
            } else if status_code == 403 {
                RestApiError::Waf
            } else if status_code >= 400 {
                RestApiError::ClientError(resp.text().await.unwrap_or_default())
            } else {
                RestApiError::Unknown(resp.text().await.unwrap_or_default())
            };
            Err(e)
        } else {
            Ok(resp)
        }
    }

    /// REST请求，返回响应的Body字符串，否则报错
    ///```rust
    ///use ba_api::KLineInterval;
    ///use ba_api::client::params;
    ///use ba_api::client::RestConn;
    ///
    ///let api_key = "TAfI8PqyqOYiegSoijSy";
    ///let sec_key = "MOIeI2mK13IEyrLHfCwh";
    ///let rest_conn = RestConn::new(api_key, sec_key, Some("http://127.0.0.1:8118"));
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
        let mut url = self.base_url.join(path).expect("invalid url");

        match params.check_type() {
            CheckType::None | CheckType::UserStream | CheckType::MarketData => {
                let query = serde_urlencoded::to_string(params).unwrap();
                url.set_query(Some(&query));
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

        let mut resp = match RestMethod::from_str(method)? {
            RestMethod::Get => self.conn.get(url),
            RestMethod::Post => self.conn.post(url),
            RestMethod::Put => self.conn.put(url),
            RestMethod::Delete => self.conn.delete(url),
        }
        .send()
        .await?;

        resp = Self::check_rest_resp(resp).await?;

        Ok(resp.text().await.unwrap())
    }
}
