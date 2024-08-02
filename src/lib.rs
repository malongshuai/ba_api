pub mod client;
pub mod errors;
pub mod misc;

pub use ba_types::*;
pub use chrono_ext::*;
pub use misc::*;

use std::path::PathBuf;

/// Rest接口的BASE URL
pub const REST_BASE_URL: &str = "https://api.binance.com";
/// WebSocket接口的BASE URL
pub const WS_BASE_URL: &str = "wss://stream.binance.com:9443";
pub const WS_BASE_URL1: &str = "wss://stream.binance.com:443";

/// 程序目录($HOME/ba_app)
pub fn app_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    let path = std::path::Path::new(&home).join("ba_app").to_path_buf();
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
/// 运行时目录($HOME/ba_app/bian)
pub fn runtime_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = app_dir()?.join("bian");
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
