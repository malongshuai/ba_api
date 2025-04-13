/// REST客户端连接
pub mod rest;

/// WebSocket客户端连接
#[cfg(feature = "websocket")]
pub mod websocket;

/// Rest请求的参数和签名鉴权类型
///
/// 所有请求都需要实现Serialize和Param Trait，
/// 如果请求的参数为空，则定义为空的Struct并实现这两个Trait
pub mod params;

/// [行情接口](rest/struct.RestConn.html#impl-1)，币安API Doc行情接口下的方法都在此
pub mod market_data;

/// [现货交易接口](rest/struct.RestConn.html#impl-2)，币安API Doc现货账户和现货交易接口下的方法都在此
pub mod spot_account_trade;

/// 子账户
pub mod sub_account;

/// 钱包相关接口
pub mod wallet;

/// 限速规则
// pub mod rate_limit;
pub(crate) mod rate_limit;
pub mod signature;
// pub mod websocket1;

pub use rest::*;
#[cfg(feature = "websocket")]
pub use websocket::*;

