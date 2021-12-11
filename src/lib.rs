pub mod base;
// pub mod coin;
// pub mod config;
pub mod client;
pub mod errors;
pub mod types;

// https://api.binance.com/api -> https://testnet.binance.vision/api
// https://api1.binance.com/api
// https://api2.binance.com/api
// https://api3.binance.com/api
// wss://stream.binance.com:9443/ws -> wss://testnet.binance.vision/ws
// wss://stream.binance.com:9443/stream -> wss://testnet.binance.vision/stream

pub use types::kline_interval::KLineInterval;
pub use types::kline::KLine;
pub use types::kline::KLines;




#[cfg(test)]
mod tests {
}
