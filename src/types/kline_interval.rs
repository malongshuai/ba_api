use std::fmt;

use serde::{Deserialize, Serialize};

/// K线间隔，包括：1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KLineInterval {
    /// 1分钟间隔
    #[serde(rename = "1m")]
    Min1 = 60,
    /// 3分钟间隔
    #[serde(rename = "3m")]
    Min3 = 180,
    /// 5分钟间隔
    #[serde(rename = "5m")]
    Min5 = 300,
    /// 15分钟间隔
    #[serde(rename = "15m")]
    Min15 = 900,
    /// 30分钟间隔
    #[serde(rename = "30m")]
    Min30 = 1800,
    /// 1小时间隔
    #[serde(rename = "1h")]
    Hour1 = 3600,
    /// 2小时间隔
    #[serde(rename = "2h")]
    Hour2 = 7200,
    /// 4小时间隔
    #[serde(rename = "4h")]
    Hour4 = 14400,
    /// 6小时间隔
    #[serde(rename = "6h")]
    Hour6 = 21600,
    /// 8小时间隔
    #[serde(rename = "8h")]
    Hour8 = 28800,
    /// 12小时间隔
    #[serde(rename = "12h")]
    Hour12 = 43200,
    /// 1天间隔
    #[serde(rename = "1d")]
    Day1 = 86400,
    /// 3天间隔
    #[serde(rename = "3d")]
    Day3 = 3 * 86400,
    /// 1周间隔
    #[serde(rename = "1w")]
    Week1 = 7 * 86400,
    /// 1月间隔
    #[serde(rename = "1M")]
    Mon1,
}

impl KLineInterval {
    pub fn is_intv(interval: &str) -> bool {
        let valid_interval = [
            "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d", "3d", "1w",
            "1M",
        ];
        valid_interval.contains(&interval)
    }
}

impl fmt::Display for KLineInterval {
    /// ```rust
    /// use ba_api::KLineInterval;
    /// let s = KLineInterval::Min3.to_string();
    /// assert_eq!(s, "3m");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Min1 => write!(f, "1m"),
            Self::Min3 => write!(f, "3m"),
            Self::Min5 => write!(f, "5m"),
            Self::Min15 => write!(f, "15m"),
            Self::Min30 => write!(f, "30m"),
            Self::Hour1 => write!(f, "1h"),
            Self::Hour2 => write!(f, "2h"),
            Self::Hour4 => write!(f, "4h"),
            Self::Hour6 => write!(f, "6h"),
            Self::Hour8 => write!(f, "8h"),
            Self::Hour12 => write!(f, "12h"),
            Self::Day1 => write!(f, "1d"),
            Self::Day3 => write!(f, "3d"),
            Self::Week1 => write!(f, "1w"),
            Self::Mon1 => write!(f, "1M"),
        }
    }
}

impl From<&str> for KLineInterval {
    ///```rust
    /// use ba_api::KLineInterval;
    /// assert_eq!(KLineInterval::Min1, KLineInterval::from("1m"));
    ///```
    fn from(s: &str) -> Self {
        match s {
            "1m" => Self::Min1,
            "3m" => Self::Min3,
            "5m" => Self::Min5,
            "15m" => Self::Min15,
            "30m" => Self::Min30,
            "1h" => Self::Hour1,
            "2h" => Self::Hour2,
            "4h" => Self::Hour4,
            "6h" => Self::Hour6,
            "8h" => Self::Hour8,
            "12h" => Self::Hour12,
            "1d" => Self::Day1,
            "3d" => Self::Day3,
            "1w" => Self::Week1,
            "1M" => Self::Mon1,
            _ => panic!("unsupported kline interval"),
        }
    }
}

#[cfg(test)]
mod kline_interval_test {
    use super::*;

    #[test]
    fn interval_test() {
        let mut s = KLineInterval::Min1.to_string();
        assert_eq!(s, "1m");

        s = KLineInterval::Min3.to_string();
        assert_eq!(s, "3m");

        s = KLineInterval::Min5.to_string();
        assert_eq!(s, "5m");

        s = KLineInterval::Min15.to_string();
        assert_eq!(s, "15m");

        s = KLineInterval::Min30.to_string();
        assert_eq!(s, "30m");

        s = KLineInterval::Hour1.to_string();
        assert_eq!(s, "1h");

        s = KLineInterval::Hour2.to_string();
        assert_eq!(s, "2h");

        s = KLineInterval::Hour4.to_string();
        assert_eq!(s, "4h");

        s = KLineInterval::Hour6.to_string();
        assert_eq!(s, "6h");

        s = KLineInterval::Hour8.to_string();
        assert_eq!(s, "8h");

        s = KLineInterval::Hour12.to_string();
        assert_eq!(s, "12h");

        s = KLineInterval::Day1.to_string();
        assert_eq!(s, "1d");

        s = KLineInterval::Day3.to_string();
        assert_eq!(s, "3d");

        s = KLineInterval::Week1.to_string();
        assert_eq!(s, "1w");

        s = KLineInterval::Mon1.to_string();
        assert_eq!(s, "1M");
    }
}
