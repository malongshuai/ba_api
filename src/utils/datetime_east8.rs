use chrono::{DateTime, FixedOffset, Utc};

/// 东八区
pub fn east8() -> FixedOffset {
    FixedOffset::east(8 * 3600)
}

/// Utc+00:00的时间
pub fn now0() -> DateTime<Utc> {
    Utc::now()
}

/// 东八区(UTC+08:00)的时间
pub fn now8() -> DateTime<FixedOffset> {
    now0().with_timezone(&east8())
}

/// Utc+00:00的凌晨
pub fn today0() -> DateTime<Utc> {
    now0().date().and_hms(0, 0, 0)
}

/// Utc+00:00的昨日凌晨
pub fn yestoday0() -> DateTime<Utc> {
    (now0() - chrono::Duration::days(1)).date().and_hms(0, 0, 0)
}

/// 东八区(UTC+08:00)的凌晨
pub fn today8() -> DateTime<FixedOffset> {
    now8().date().and_hms(0, 0, 0)
}
/// 东八区(UTC+08:00)的昨日凌晨
pub fn yestoday8() -> DateTime<FixedOffset> {
    (now8() - chrono::Duration::days(1)).date().and_hms(0, 0, 0)
}
