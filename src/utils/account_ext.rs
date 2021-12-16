use crate::{Account, Balances, RawBalance};

/// 为账户提供一些便捷的方法
pub trait AccountExt {
    /// maker手续费
    fn maker_fee(&self) -> f64;

    /// 返回某个币的余额信息，包含已冻结和空闲可用的余额，返回值`(free, locked)`
    fn coin_balance(&self, coin: &str) -> RawBalance;

    /// 返回某个币空闲可用的资产余额
    fn coin_balance_free(&self, coin: &str) -> f64 {
        self.coin_balance(coin).free
    }

    /// 返回某个币已冻结的资产余额
    fn coin_balance_locked(&self, coin: &str) -> f64 {
        self.coin_balance(coin).locked
    }
}

impl AccountExt for Account {
    /// maker手续费(如0.1%，将返回0.001)
    fn maker_fee(&self) -> f64 {
        // maker_fee是手续费，其值为u16，数值10表示手续费为0.1%(10 * 0.0001)
        self.maker_fee as f64 / 10000f64
    }

    /// 返回某个币的余额信息，包含已冻结和空闲可用的余额
    fn coin_balance(&self, coin: &str) -> RawBalance {
        self.balances.coin_balance(coin)
    }
}

/// 为获取账户余额提供一些便捷的方法
pub trait BalancesExt {
    /// 返回某个币的余额信息，包含已冻结和空闲可用的余额，返回值`(free, locked)`
    fn coin_balance(&self, coin: &str) -> RawBalance;

    /// 返回某个币空闲可用的资产余额
    fn coin_balance_free(&self, coin: &str) -> f64 {
        self.coin_balance(coin).free
    }

    /// 返回某个币已冻结的资产余额
    fn coin_balance_locked(&self, coin: &str) -> f64 {
        self.coin_balance(coin).locked
    }
}

impl BalancesExt for Balances {
    /// 返回某个币的余额信息，包含已冻结和空闲可用的余额，返回值`(free, locked)`
    fn coin_balance(&self, coin: &str) -> RawBalance {
        let mut res = RawBalance::new();
        if let Some(d) = self.0.get(coin) {
            res.free = d.free;
            res.locked = d.locked;
        }

        res
    }
}
