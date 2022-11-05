pub mod client;
pub mod errors;
pub mod misc;
pub mod types;
pub mod utils;

pub use misc::check_coin_warning;
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
pub use utils::{align_epoch, east8, now0, now8, today0, today8, yestoday0, yestoday8};

pub const STABLE_COINS: [&str; 21] = [
    "AUD", "BIDR", "BRL", "EUR", "GBP", "RUB", "TRY", "TUSD", "USDC", "DAI", "IDRT", "UAH", "NGN",
    "VAI", "USDP", "PAX", "SUSD", "BVND", "BUSD", "UST", "PAXG",
];

/// Rest接口的BASE URL
pub const REST_BASE_URL: &str = "https://api.binance.com";
/// WebSocket接口的BASE URL
pub const WS_BASE_URL: &str = "wss://stream.binance.com:9443";
