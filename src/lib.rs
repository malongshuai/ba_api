use std::env;

pub mod client;
pub mod errors;
pub mod types;
pub mod utils;

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

pub const STABLE_COINS: [&str; 20] = [
    "AUD", "BIDR", "BRL", "EUR", "GBP", "RUB", "TRY", "TUSD", "USDC", "DAI", "IDRT", "UAH", "NGN",
    "VAI", "USDP", "PAX", "SUSD", "BVND", "BUSD", "UST"
];

#[macro_use]
extern crate lazy_static;

lazy_static! {
    /// Rest接口的BASE URL，`RUN_MODE`环境变量设置为`test`时，自动进入币安的测试模式，参考<https://testnet.binance.vision/>
    pub static ref REST_BASE_URL: &'static str = match env::var("RUN_MODE") {
      Err(_) => "https://api.binance.com",
      Ok(v) if v == "test" => "https://testnet.binance.vision",
      Ok(_) => panic!("Value of environment variable `RUN_MODE` must be `test` or omitted"),
    };

    /// WebSocket接口的BASE URL，`RUN_MODE`环境变量设置为`test`时，自动进入币安的测试模式，参考<https://testnet.binance.vision/>
    pub static ref WS_BASE_URL: &'static str = match env::var("RUN_MODE") {
        Err(_) => "wss://stream.binance.com:9443",
        Ok(v) if v == "test" => "wss://testnet.binance.vision",
        Ok(_) => panic!("Value of environment variable `RUN_MODE` must be `test` or omitted"),
    };
}
