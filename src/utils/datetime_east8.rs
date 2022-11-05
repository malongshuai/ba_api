use chrono::{DateTime, FixedOffset, TimeZone, Timelike, Utc};

use crate::KLineInterval;

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

/// 给定一个时间，根据目标K线间隔类型，找到对齐的起始点
pub fn align_epoch(epoch: u64, dest_type: KLineInterval) -> u64 {
    // 因会频繁调取该方法，不要使用chrono::Local，Local相关时间的操作效率远远低于指定时区的方式
    let dt = east8().timestamp_millis(epoch as i64);

    let (h, m) = (dt.hour(), dt.minute());
    //&-------- 注： chrono的with_xxx()(例如with_hour())效率不高
    //&-------- 使用date().and_hms()替代
    let aligned_dt = match dest_type {
        KLineInterval::Min1 => dt,
        KLineInterval::Min3 => dt.date().and_hms(h, m - (m % 3), 0),
        KLineInterval::Min5 => dt.date().and_hms(h, m - (m % 5), 0),
        KLineInterval::Min15 => dt.date().and_hms(h, m - (m % 15), 0),
        KLineInterval::Min30 => dt.date().and_hms(h, m - (m % 30), 0),
        KLineInterval::Hour1 => dt.date().and_hms(h, 0, 0),
        KLineInterval::Hour2 => dt.date().and_hms(h - (h % 2), 0, 0),
        KLineInterval::Hour4 => dt.date().and_hms(h - (h % 4), 0, 0),
        KLineInterval::Hour6 => dt.date().and_hms(h - (h % 6), 0, 0),
        KLineInterval::Hour8 => dt.date().and_hms(h - (h % 8), 0, 0),
        KLineInterval::Hour12 => dt.date().and_hms(h - (h % 12), 0, 0),
        KLineInterval::Day1 => dt.date().and_hms(0, 0, 0),
        _ => panic!("unsupport dest_interval: {}", dest_type),
    };
    aligned_dt.timestamp_millis() as u64
}
