pub trait FloatPercent {
    fn percent(&self, n: f64) -> f64;
}

impl FloatPercent for f64 {
    #[inline]
    fn percent(&self, n: f64) -> f64 {
        (n - self) / self
    }
}

/// 将浮点数截断到指定小数位数，例如`123.45678.truncate(3) == 123.456`
pub trait FloatTruncate {
    fn truncate(&self, len: u8) -> f64;
}

impl FloatTruncate for f64 {
    /// 将浮点数截断到指定小数位数，例如`123.45678.truncate(3) == 123.456`
    fn truncate(&self, len: u8) -> f64 {
        let l = len.into();
        (self * 10f64.powi(len.into())).floor().trunc() / 10f64.powi(l)
    }
}

/// 计算浮点数精度的位数，例如`0.0023000f64.precision() == 4`
pub trait FloatPrecision {
    fn precision(&self) -> u8;
}

impl FloatPrecision for f64 {
    /// 计算浮点数精度的位数，例如`0.0023000f64.precision() == 4`
    fn precision(&self) -> u8 {
        let i = self.trunc();
        let frac = self - i;
        if (frac - 0.0).abs() < f64::EPSILON {
            return 0u8;
        }
        (frac.to_string().len() - 2) as u8
    }
}

#[cfg(test)]
mod base_test {
    use super::*;

    #[test]
    fn test_precision() {
        assert_eq!(0.1.precision(), 1u8);
        assert_eq!(0f64.precision(), 0u8);
        assert_eq!(0.000020000.precision(), 5u8);
        assert_eq!(0.0000001.precision(), 7u8);
        assert_eq!(0.000000.precision(), 0u8);
    }

    #[test]
    fn test_truncate() {
        let error_margin = f64::EPSILON;
        assert!((0f64.truncate(0) - 0f64).abs() < error_margin);
        assert!((1234f64.truncate(0) - 1234f64).abs() < error_margin);
        assert!((1234.34567f64.truncate(0) - 1234f64).abs() < error_margin);
        assert!((1234.34567f64.truncate(1) - 1234.3f64).abs() < error_margin);
        assert!((1234.34567f64.truncate(3) - 1234.345f64).abs() < error_margin);
    }
}
