use crate::base::{FloatPrecision, ToSymbol};
use crate::rest;
use serde::Deserialize;
use serde_json;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;

pub const STABLE_COINS: [&str; 19] = [
    "AUD", "BRL", "EUR", "GBP", "RUB", "TRY", "DAI", "UAH", "NGN", "PAX", "VAI", "SUSD", "BVND",
    "USDP", "IDRT", "BUSD", "BIDR", "TUSD", "USDC",
];

pub async fn get_all_symbols_full_info(conn: reqwest::Client) -> Option<AllSymbolsFullInfo> {
    let exchange_path = "/api/v3/exchangeInfo";

    match rest::rest_req(conn, exchange_path, "").await {
        None => None,
        Some(body) => match serde_json::from_str::<ExchangeInfo>(&body) {
            Err(_) => None,
            Ok(exchange_info) => Some(exchange_info.all_symbols_info()),
        },
    }
}

/// 获取所有有效的USDT币种
pub async fn get_all_usdt_symbols(conn: reqwest::Client) -> Option<AllUsdtSymbols> {
    match get_all_symbols_full_info(conn).await {
        None => None,
        Some(all_symbols_full_info) => Some(all_symbols_full_info.all_valid_usdt_symbols()),
    }
}

/// 获取给定币种当前的ticker，即当前挂单的最低卖价和数量、最高买价和数量
pub async fn coin_ticker(conn: reqwest::Client, coin: &str) -> Option<Ticker> {
    let ticker_path = "/api/v3/ticker/bookTicker";
    let params = format!("symbol={}", coin.to_symbol());
    match rest::rest_req(conn, ticker_path, &params).await {
        None => None,
        Some(body) => match serde_json::from_str::<Ticker>(&body) {
            Err(_) => None,
            Ok(ticker) => Some(ticker),
        },
    }
}

/// 从官方请求所有交易对信息，序列化后保存在此结构中
#[derive(Deserialize)]
pub struct ExchangeInfo {
    // 将json字符串中的symbols字段的值，序列化为AllSymbolsFullInfo结构
    #[serde(flatten)]
    symbols: AllSymbolsFullInfo,
}

impl ExchangeInfo {
    pub fn all_symbols_info(self) -> AllSymbolsFullInfo {
        self.symbols
    }
}

#[derive(Deserialize)]
pub struct AllSymbolsFullInfo {
    // 将json字符串中的symbols字段识别为该字段名(all_symbols_info)
    // #[serde(rename = "symbols")]
    // all_symbols_info: Vec<SymbolFullInfo>,
    symbols: Vec<SymbolFullInfo>,
}

impl AllSymbolsFullInfo {
    pub fn all_valid_usdt_symbols(&self) -> AllUsdtSymbols {
        let mut usdt_symbols = AllUsdtSymbols {
            symbols: HashMap::new(),
            symbol_names: vec![],
        };

        for info in self.symbols.iter() {
            if info.is_valid_usdt_symbol() {
                let symbol_name = info.symbol().to_string();
                usdt_symbols
                    .symbols
                    .insert(symbol_name.clone(), info.to_symbol());
                usdt_symbols.symbol_names.push(symbol_name);
            }
        }
        usdt_symbols
    }
}

#[derive(Deserialize, Debug)]
pub struct AllUsdtSymbols {
    symbols: HashMap<String, Symbol>,
    symbol_names: Vec<String>,
}

impl AllUsdtSymbols {
    pub fn count(&self) -> usize {
        self.symbol_names.len()
    }

    pub fn get(&self, symbol_name: &str) -> Option<&Symbol> {
        self.symbols.get(symbol_name)
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub struct SymbolFullInfo {
    symbol: String,
    status: String,
    base_asset: String,
    base_asset_precision: u8,
    quote_asset: String,
    quote_precision: u8,
    quote_asset_precision: u8,
    base_commission_precision: u8,
    quote_commission_precision: u8,
    order_types: Vec<String>,
    iceberg_allowed: bool,
    oco_allowed: bool,
    quote_order_qty_market_allowed: bool,
    is_spot_trading_allowed: bool,
    is_margin_trading_allowed: bool,
    filters: Vec<HashMap<String, serde_json::Value>>,
    // [{filterType: "PRICE_FILTER", : minPrice: "0.01000000", : maxPrice: "1000000.00000000", : tickSize: "0.01000000"},
    //  {filterType: "PERCENT_PRICE", : multiplierUp: "5", : multiplierDown: "0.2", : avgPriceMins: 5},
    //  {filterType: "LOT_SIZE", : minQty: "0.00001000", : maxQty: "9000.00000000", : stepSize: "0.00001000"},
    //  {filterType: "MIN_NOTIONAL", : minNotional: "10.00000000", : applyToMarket: true, : avgPriceMins: 5},
    //  {filterType: "ICEBERG_PARTS", : limit: 10},
    //  {filterType: "MARKET_LOT_SIZE", : minQty: "0.00000000", : maxQty: "99.87711898", : stepSize: "0.00000000"},
    //  {filterType: "MAX_NUM_ORDERS", : maxNumOrders: 200},
    //  {filterType: "MAX_NUM_ALGO_ORDERS", : maxNumAlgoOrders: 5}],
    permissions: Vec<String>,

    #[serde(default)]
    price_precision: RefCell<u8>,

    #[serde(default)]
    amount_precision: RefCell<u8>,
}

impl SymbolFullInfo {
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn base_asset(&self) -> &str {
        &self.base_asset
    }

    pub fn quote_asset(&self) -> &str {
        &self.quote_asset
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn is_spot_trade(&self) -> bool {
        self.is_spot_trading_allowed
    }

    fn symbol_precision(&self) -> (u8, u8) {
        let mut res = (0, 0);
        for item in self.filters.iter() {
            let v = item.get("filterType").unwrap();
            if *v == serde_json::json!("PRICE_FILTER") {
                let price_precision = item
                    .get("minPrice")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
                    .precision();
                res.0 = price_precision;
            } else if *v == serde_json::json!("LOT_SIZE") {
                let amount_precision = item
                    .get("minQty")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
                    .precision();
                res.1 = amount_precision;
            }
        }
        res
    }

    pub fn price_precision(&self) -> u8 {
        let mut prec = *self.amount_precision.borrow_mut();
        if prec == 0 {
            let (p, _) = self.symbol_precision();
            prec = p;
        }
        prec
    }

    pub fn amount_precision(&self) -> u8 {
        let mut prec = *self.amount_precision.borrow_mut();
        if prec == 0 {
            let (_, a) = self.symbol_precision();
            prec = a;
        }
        prec
    }

    pub fn to_symbol(&self) -> Symbol {
        Symbol {
            name: self.symbol.clone(),
            price_prec: self.price_precision(),
            amount_prec: self.amount_precision(),
        }
    }

    pub fn is_valid_usdt_symbol(&self) -> bool {
        if self.quote_asset != "USDT" || self.status != "TRADING" || !self.is_spot_trading_allowed {
            return false;
        }

        if self.base_asset.ends_with("DOWN")
            || self.base_asset.ends_with("UP")
            || self.base_asset.ends_with("BULL")
            || self.base_asset.ends_with("BEAR")
        {
            return false;
        }

        if STABLE_COINS.contains(&&*self.base_asset) {
            return false;
        }

        true
    }
}

#[derive(Deserialize, Default, PartialEq, Debug)]
pub struct Symbol {
    name: String,
    price_prec: u8,
    amount_prec: u8,
}

impl Symbol {
    pub fn to_str(&self) -> &str {
        &self.name
    }
    pub fn price_precision(&self) -> u8 {
        self.price_prec
    }
    pub fn amount_precision(&self) -> u8 {
        self.amount_prec
    }
}

impl ToString for Symbol {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Ticker {
    symbol: String,
    bid_price: String,
    bid_qty: String,
    ask_price: String,
    ask_qty: String,
}

impl Ticker {
    pub fn bid_price(&self) -> f64 {
        self.bid_price.parse::<f64>().unwrap()
    }
    pub fn bid_qty(&self) -> f64 {
        self.bid_qty.parse::<f64>().unwrap()
    }
    pub fn ask_price(&self) -> f64 {
        self.ask_price.parse::<f64>().unwrap()
    }
    pub fn ask_qty(&self) -> f64 {
        self.ask_qty.parse::<f64>().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::ExchangeInfo;
    use crate::coin::Symbol;
    use serde_json;

    fn data_source() -> &'static str {
        r#"
          {
          "timezone": "UTC",
          "serverTime": 1634895754487,
          "rateLimits": [{
              "rateLimitType": "REQUEST_WEIGHT",
              "interval": "MINUTE",
              "intervalNum": 1,
              "limit": 1200
            }, {
              "rateLimitType": "ORDERS",
              "interval": "SECOND",
              "intervalNum": 10,
              "limit": 50
            }
          ],
          "exchangeFilters": [],
          "symbols": [{
              "symbol": "ETHBTC",
              "status": "TRADING",
              "baseAsset": "ETH",
              "baseAssetPrecision": 8,
              "quoteAsset": "BTC",
              "quotePrecision": 8,
              "quoteAssetPrecision": 8,
              "baseCommissionPrecision": 8,
              "quoteCommissionPrecision": 8,
              "orderTypes": ["LIMIT", "LIMIT_MAKER", "MARKET", "STOP_LOSS_LIMIT", "TAKE_PROFIT_LIMIT"],
              "icebergAllowed": true,
              "ocoAllowed": true,
              "quoteOrderQtyMarketAllowed": true,
              "isSpotTradingAllowed": true,
              "isMarginTradingAllowed": true,
              "filters": [{
                  "filterType": "PRICE_FILTER",
                  "minPrice": "0.00000100",
                  "maxPrice": "922327.00000000",
                  "tickSize": "0.00000100"
                }, {
                  "filterType": "PERCENT_PRICE",
                  "multiplierUp": "5",
                  "multiplierDown": "0.2",
                  "avgPriceMins": 5
                }, {
                  "filterType": "LOT_SIZE",
                  "minQty": "0.00010000",
                  "maxQty": "100000.00000000",
                  "stepSize": "0.00010000"
                }, {
                  "filterType": "MIN_NOTIONAL",
                  "minNotional": "0.00010000",
                  "applyToMarket": true,
                  "avgPriceMins": 5
                }
              ],
              "permissions": ["SPOT", "MARGIN"]
            }, {
              "symbol": "LTCUSDT",
              "status": "TRADING",
              "baseAsset": "LTC",
              "baseAssetPrecision": 8,
              "quoteAsset": "USDT",
              "quotePrecision": 8,
              "quoteAssetPrecision": 8,
              "baseCommissionPrecision": 8,
              "quoteCommissionPrecision": 8,
              "orderTypes": ["LIMIT", "LIMIT_MAKER", "MARKET", "STOP_LOSS_LIMIT", "TAKE_PROFIT_LIMIT"],
              "icebergAllowed": true,
              "ocoAllowed": true,
              "quoteOrderQtyMarketAllowed": true,
              "isSpotTradingAllowed": true,
              "isMarginTradingAllowed": true,
              "filters": [{
                  "filterType": "PRICE_FILTER",
                  "minPrice": "0.00000100",
                  "maxPrice": "100000.00000000",
                  "tickSize": "0.00000100"
                }, {
                  "filterType": "PERCENT_PRICE",
                  "multiplierUp": "5",
                  "multiplierDown": "0.2",
                  "avgPriceMins": 5
                }, {
                  "filterType": "LOT_SIZE",
                  "minQty": "0.00100000",
                  "maxQty": "100000.00000000",
                  "stepSize": "0.00100000"
                }, {
                  "filterType": "MIN_NOTIONAL",
                  "minNotional": "0.00010000",
                  "applyToMarket": true,
                  "avgPriceMins": 5
                }
              ],
              "permissions": ["SPOT", "MARGIN"]
            }]}"#
    }

    #[test]
    fn test_deserialize() -> Result<(), serde_json::Error> {
        let json_str = data_source();
        serde_json::from_str::<ExchangeInfo>(json_str)?;

        Ok(())
    }

    #[test]
    fn test_all_usdt_symbols() {
        let json_str = data_source();

        let exchange_info = serde_json::from_str::<ExchangeInfo>(json_str).unwrap();
        let all_symbols_info = exchange_info.all_symbols_info();
        let all_usdt_symbols = all_symbols_info.all_valid_usdt_symbols();

        let ltcusdt = Symbol {
            name: "LTCUSDT".to_string(),
            price_prec: 6,
            amount_prec: 3,
        };

        assert_eq!(all_usdt_symbols.get("LTCUSDT"), Some(&ltcusdt));
        assert_eq!(all_usdt_symbols.get("BTCUSDT"), None);
        assert_eq!(all_usdt_symbols.count(), 1);
    }
}
