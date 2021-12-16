use crate::KLine;

use super::FloatPercent;

pub trait KLineExt {
    /// (close - open) / open
    fn rate(&self) -> f64;
}

impl KLineExt for KLine {
    /// (close - open) / open
    fn rate(&self) -> f64 {
        self.open.percent(self.close)
    }
}
