use std::fmt::Display;
use thiserror::Error;
use tokio_tungstenite::tungstenite;

#[derive(Debug, Error)]
pub enum RestApiError {
    #[error("4xx client error: {0}")]
    ClientError(String),

    #[error("5xx server error: {0}")]
    ServerError(String),

    #[error("403 waf error")]
    Waf,

    #[error("429 waf warning")]
    WafWarning,

    #[error("418 waf blocked")]
    Blocked,

    #[error(transparent)]
    RequestError(#[from] reqwest::Error),

    #[error(transparent)]
    MethodError(#[from] MethodError),

    #[error(transparent)]
    DecodeError(#[from] serde_json::Error),

    #[error("argument error: {0}")]
    ArgumentError(String),

    #[error("unknown error: {0}")]
    Unknown(String),
}

pub type RestResult<T> = Result<T, RestApiError>;

#[derive(Debug, Error)]
pub struct MethodError {
    pub msg: String,
}

impl Display for MethodError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unsupported method: `{}', supports: get/put/post/delete",
            self.msg
        )
    }
}

#[derive(Debug, Error)]
pub enum WsApiError {
    #[error(transparent)]
    WsError(#[from] tungstenite::error::Error),

    /// 将要订阅的数量超出了限制(币安每个ws连接最多只允许定于1024个Stream)
    #[error("too many subscribes {0}")]
    TooManySubscribes(usize),
}
pub type WsResult<T> = Result<T, WsApiError>;
