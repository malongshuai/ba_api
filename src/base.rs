pub trait Truncate {
    /// 123.45678.truncate(3) == 123.456
    fn truncate(&self, len: u8) -> f64;
}

impl Truncate for f64 {
    fn truncate(&self, len: u8) -> f64 {
        let l = len.into();
        (self * 10f64.powi(len.into())).floor().trunc() / 10f64.powi(l)
    }
}

pub trait FloatPrecision {
    fn precision(&self) -> u8;
}

impl FloatPrecision for String {
    fn precision(&self) -> u8 {
        let f = self.parse::<f64>().unwrap();
        let i = f.trunc();
        let frac = f - i;
        if frac == 0.0 {
            return 0u8;
        }
        (frac.to_string().len() - 2) as u8
    }
}

pub trait ToSymbol {
    /// String or &str -> uppercase Symbol String,
    /// "btc", "BTC", "btcusdt", "BTCUSDT" all convert to "BTCUSDT"
    fn to_symbol(&self) -> String;
}

impl<T> ToSymbol for T
where T: ToString
{
    fn to_symbol(&self) -> String {
        let res = self.to_string().to_uppercase();
        if res.ends_with("USDT") {
            res
        } else {
            format!("{}USDT", res)
        }
    }
}

#[cfg(test)]
mod base_test {
    use super::*;

    #[test]
    fn test_precision() {
        assert_eq!("0.1".to_string().precision(), 1u8);
        assert_eq!("0".to_string().precision(), 0u8);
        assert_eq!("0.000020000".to_string().precision(), 5u8);
        assert_eq!("0.0000001".to_string().precision(), 7u8);
        assert_eq!("0.000000".to_string().precision(), 0u8);
    }

    #[test]
    fn test_truncate() {
        let error_margin = f64::EPSILON;
        assert!((0f64.truncate(0) - 0f64).abs() < error_margin);
        assert!((1234f64.truncate(0) -  1234f64).abs() < error_margin);
        assert!((1234.34567f64.truncate(0) - 1234f64).abs() < error_margin);
        assert!((1234.34567f64.truncate(1) - 1234.3f64).abs() < error_margin);
        assert!((1234.34567f64.truncate(3) - 1234.345f64).abs() < error_margin);
    }

    #[test]
    fn test_to_symbol() {
        assert_eq!("btc".to_symbol(), "BTCUSDT".to_string());
        assert_eq!("BTC".to_symbol(), "BTCUSDT".to_string());
        assert_eq!("BTCusdt".to_symbol(), "BTCUSDT".to_string());
        assert_eq!("BTCUSDT".to_string().to_symbol(), "BTCUSDT".to_string());
    }
}
