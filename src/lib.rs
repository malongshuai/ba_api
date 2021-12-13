pub mod client;
pub mod errors;
pub mod types;

pub use types::account::*;
pub use types::depth::*;
pub use types::kline::*;
pub use types::kline_interval::KLineInterval;
pub use types::order::*;
pub use types::other_types::*;
pub use types::rate_limit::*;
pub use types::symbol_info::*;
pub use types::ticker::*;
pub use types::ws_response::*;

/// 币安REST接口的base url
pub const REST_BASE_URL: &str = "https://api.binance.com";
/// 币安REST接口的测试base url
pub const REST_TEST_BASE_URL: &str = "https://testnet.binance.vision";
/// 币安WS接口的base url
pub const WS_BASE_URL: &str = "wss://stream.binance.com:9443";
/// 币安WS接口的测试base url
pub const WS_TEST_BASE_URL: &str = "wss://testnet.binance.vision";

