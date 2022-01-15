use crate::{KLine, KLineInterval};

use super::FloatPercent;

pub trait KLineExt {
    /// (close - open) / open
    fn rate(&self) -> f64;

    /// 根据多根K线合并成一根K线，K线类型由第一根K线决定，K线是否完成，由最后一根K线决定
    fn merge_from_ks(ks: Vec<&KLine>) -> KLine;

    /// 从给定的字符串组成K线  
    /// 
    /// 字段顺序:  
    /// 
    /// symbol,intv_type,epoch,close_epoch,finish,open,high,low,close,amount,vol,count
    /// 
    /// 例如：  
    /// 
    /// 1INCHUSDT,1m,1642039980000,1642040039999,true,2.446,2.458,2.444,2.453,58423.5,143086.3826,621
    fn from_str(kline_str: &str) -> Option<KLine>;
}

impl KLineExt for KLine {
    /// (close - open) / open
    fn rate(&self) -> f64 {
        self.open.percent(self.close)
    }

    /// 根据多根K线合并成一根K线，K线类型由第一根K线决定，K线是否完成，由最后一根K线决定
    fn merge_from_ks(ks: Vec<&KLine>) -> KLine {
        let mut merged_kl = ks[0].clone();
        let last_kl = &ks[ks.len() - 1];
        merged_kl.close_epoch = last_kl.close_epoch;
        merged_kl.close = last_kl.close;
        merged_kl.finish = last_kl.finish;

        for k in &ks[1..] {
            if k.high > merged_kl.high {
                merged_kl.high = k.high
            }
            if k.low < merged_kl.low {
                merged_kl.low = k.low
            }
            merged_kl.count += k.count;
            merged_kl.vol += k.vol;
            merged_kl.amount += k.amount;
        }
        merged_kl
    }

    fn from_str(kline_str: &str) -> Option<KLine> {
        let s: Vec<&str> = kline_str.split(',').collect();
        Some(KLine {
            symbol: s[0].to_string(),
            interval: KLineInterval::from(s[1]),
            epoch: s[2].parse().ok()?,
            close_epoch: s[3].parse().ok()?,
            finish: s[4].parse().ok()?,
            open: s[5].parse().ok()?,
            high: s[6].parse().ok()?,
            low: s[7].parse().ok()?,
            close: s[8].parse().ok()?,
            amount: s[9].parse().ok()?,
            vol: s[10].parse().ok()?,
            count: s[11].parse().ok()?,
        })
    }
}
