use serde::{Deserialize, Serialize};
use std::{fmt::Display, io};
use thiserror::Error;
#[cfg(feature = "websocket")]
use tokio_tungstenite::tungstenite;

#[derive(Debug, Error)]
pub enum BiAnApiError {
    #[error("4xx client error: {0}")]
    ClientError(String),

    #[error("400 Bad Request, code: {0}, msg: {1}")]
    BadRequest(i32, String),

    #[error("5xx server error: {0}")]
    ServerError(String),

    #[error("403 waf error")]
    Waf,

    #[error("429 waf warning")]
    WafWarning,

    #[error("418 waf blocked")]
    Blocked,

    #[error("connect err to {0}")]
    ConnectError(String),

    #[error(transparent)]
    RequestError(#[from] reqwest::Error),

    #[error(transparent)]
    MethodError(#[from] MethodError),

    #[error(transparent)]
    DecodeError(#[from] serde_json::Error),

    #[error("argument error: {0}")]
    ArgumentError(String),

    /// ws错误
    #[cfg(feature = "websocket")]
    #[error(transparent)]
    WsError(#[from] tungstenite::error::Error),

    /// ws错误，将要订阅的数量超出了限制(币安每个ws连接最多只允许定于1024个Stream)
    #[error("too many subscribes {0}")]
    TooManySubscribes(usize),

    /// io错误
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("unknown error: {0}")]
    Unknown(String),
}

pub type BiAnResult<T> = Result<T, BiAnApiError>;

#[derive(Debug, Error)]
pub struct MethodError {
    pub msg: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BadRequest {
    pub code: i32,
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
