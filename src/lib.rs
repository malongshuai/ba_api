pub mod client;
pub mod errors;
pub mod misc;

pub use ba_types::*;
pub use misc::check_coin_warning;

/// Rest接口的BASE URL
pub const REST_BASE_URL: &str = "https://api.binance.com";
/// WebSocket接口的BASE URL
pub const WS_BASE_URL: &str = "wss://stream.binance.com:9443";
