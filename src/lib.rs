pub mod client;
pub mod errors;
pub mod misc;

pub use ba_types::*;
pub use chrono_ext::*;
pub use misc::*;

/// Rest接口的BASE URL
pub const REST_BASE_URL: &str = "https://api.binance.com";
/// WebSocket接口的BASE URL
pub const WS_BASE_URL: &str = "wss://stream.binance.com:9443";

/// 程序目录($HOME)
pub fn app_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    let path = std::path::Path::new(&home).join("ba_app").to_path_buf();
    std::fs::create_dir_all(&path)?;
    Ok(path)
}