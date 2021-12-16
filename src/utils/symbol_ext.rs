use crate::{ExchangeInfo, Permission, SymbolFilter, SymbolInfo, SymbolStatus, STABLE_COINS};

use super::{FloatPrecision, FloatTruncate};

/// 交易对信息提取
pub trait SymbolInfoExt {
    /// 挂单时的价格精度
    fn price_precision(&self) -> u8;
    /// 挂单时币的数量精度
    fn amount_precision(&self) -> u8;
    /// 最低交易额(例如对于BTCUSDT，要求最低要买入10 USDT的币)
    fn min_vol(&self) -> f64;
    /// 该币是否是允许交易的状态
    fn is_trading(&self) -> bool;
    /// 该币是否是允许交易前的状态
    fn is_pre_trading(&self) -> bool;
    /// 该币是否是允许现货交易
    fn allow_spot_trading(&self) -> bool;
    /// 该币是否是允许杠杆交易
    fn allow_margin_trading(&self) -> bool;

    /// 调整下单数量为符合数量限制的值，调整后的数量值可能会比原值稍稍减小不到一个精度
    fn adjust_amount(&self, amount: f64) -> f64;

    /// 调整下单价格为符合价格限制的值，调整后的价格可能会比原值稍稍减小不到一个精度
    fn adjust_price(&self, price: f64) -> f64;
}

impl SymbolInfoExt for SymbolInfo {
    /// 挂单时的价格精度
    fn price_precision(&self) -> u8 {
        let x = self.filters.iter().find_map(|x| match x {
            SymbolFilter::PriceFilter { min_price, .. } => Some(min_price.precision()),
            _ => None,
        });
        x.unwrap()
    }

    /// 挂单时币的数量精度
    fn amount_precision(&self) -> u8 {
        let x = self.filters.iter().find_map(|x| match x {
            SymbolFilter::LotSize { min_qty, .. } => Some(min_qty.precision()),
            _ => None,
        });
        x.unwrap()
    }

    /// 最低交易额(例如对于BTCUSDT，最低要买入10 USDT的币)
    fn min_vol(&self) -> f64 {
        let x = self.filters.iter().find_map(|x| match x {
            SymbolFilter::MinNotional { min_notional, .. } => Some(*min_notional),
            _ => None,
        });
        x.unwrap()
    }

    /// 该币是否是允许交易的状态
    fn is_trading(&self) -> bool {
        matches!(self.status, SymbolStatus::Trading)
    }

    /// 该币是否是允许交易前的状态
    fn is_pre_trading(&self) -> bool {
        matches!(self.status, SymbolStatus::PreTrading)
    }

    /// 该币是否是允许现货交易
    fn allow_spot_trading(&self) -> bool {
        self.permissions.contains(&Permission::Spot)
    }

    /// 该币是否是允许杠杆交易
    fn allow_margin_trading(&self) -> bool {
        self.permissions.contains(&Permission::Margin)
    }

    /// 调整下单数量为符合数量限制的值，调整后的数量值可能会比原值稍稍减小不到一个精度
    fn adjust_amount(&self, amount: f64) -> f64 {
        amount.truncate(self.amount_precision())
    }

    /// 调整下单价格为符合价格限制的值，调整后的价格可能会比原值稍稍减小不到一个精度
    fn adjust_price(&self, price: f64) -> f64 {
        price.truncate(self.price_precision())
    }
}

pub trait ExchangeInfoExt {
    fn symbol_info(&self, symbol: &str) -> Option<&SymbolInfo>;

    /// 找出所有quote_asset值为quote的允许现货交易的交易对信息，例如找出所有基于USDT的交易对信息，找出所有基于BNB的交易对信息
    fn all_symbols_based(&self, quote: &str) -> Vec<&SymbolInfo>;

    /// 找出所有quote_asset值为quote的允许现货交易的交易对名称
    fn all_symbol_names_based(&self, quote: &str) -> Vec<String> {
        self.all_symbols_based(quote)
            .iter()
            .map(|info| info.symbol.clone())
            .collect::<Vec<String>>()
    }

    /// 找出所有基于USDT的交易对信息
    fn all_usdt_symbols_info(&self) -> Vec<&SymbolInfo> {
        self.all_symbols_based("USDT")
    }

    /// 找出所有基于USDT的交易对名称
    fn all_usdt_symbol_names(&self) -> Vec<String> {
        self.all_symbol_names_based("USDT")
    }
}

impl ExchangeInfoExt for ExchangeInfo {
    /// 找出所有quote_asset值为quote的允许现货交易的交易对信息，例如找出所有基于USDT的交易对信息，找出所有基于BNB的交易对信息
    fn all_symbols_based(&self, quote: &str) -> Vec<&SymbolInfo> {
        let quote = quote.to_string();
        self.symbols
            .iter()
            .filter(|s_info| {
                let base = &s_info.base_asset;
                s_info.quote_asset == quote
                    && s_info.is_trading()
                    && s_info.allow_spot_trading()
                    && !STABLE_COINS.contains(&base.as_str())
                    && !base.ends_with("DOWN")
                    && !base.ends_with("UP")
                    && !base.ends_with("BULL")
                    && !base.ends_with("BEAR")
            })
            .collect::<Vec<&SymbolInfo>>()
    }

    fn symbol_info(&self, symbol: &str) -> Option<&SymbolInfo> {
        self.symbols.iter().find(|x| x.symbol == symbol)
    }
}
