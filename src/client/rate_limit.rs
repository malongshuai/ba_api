//! 从 RateLimit 来更新
//!
//! [{
//!    "rateLimitType": "REQUEST_WEIGHT",
//!    "interval": "MINUTE",
//!    "intervalNum": 1,
//!    "limit": 6000
//! }, {
//!    "rateLimitType": "ORDERS",
//!    "interval": "SECOND",
//!    "intervalNum": 10,
//!    "limit": 50
//! }, {
//!    "rateLimitType": "ORDERS",
//!    "interval": "DAY",
//!    "intervalNum": 1,
//!    "limit": 160000
//! }, {
//!    "rateLimitType": "RAW_REQUESTS",
//!    "interval": "MINUTE",
//!    "intervalNum": 5,
//!    "limit": 61000
//!    }
//! ]

use ba_types::{ExchangeInfo, RateLimit, RateLimitInterVal, RateLimitType};
use chrono_ext::{now0, DateTime, FixedOffset, ParseDateTimeExt, Timelike};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;

/// 发送的请求属于哪种类型的限速
#[derive(Debug)]
pub enum RateLimitParam {
    /// 该请求不是下单操作，参数值表示该请求所需权重值
    Weight(u32),
    /// 该请求是下单操作，参数值表示该请求所需权重值
    Order(u32),
    /// 该请求包含了下单操作(比如撤单再自动下单的请求)
    ///
    /// 参数值表示该请求所需权重值
    Both(u32),
}

struct RestApiRateLimitInfo {
    rate_limit: RateLimit,
    /// 剩余的次数
    remain: u32,
    /// 限速被重置的时间
    reset_datetime: DateTime<FixedOffset>,
}

impl RestApiRateLimitInfo {
    fn new(rate_limit: RateLimit) -> Self {
        Self {
            remain: rate_limit.limit,
            rate_limit,
            reset_datetime: now0(),
        }
    }

    fn reset_permits(&mut self) {
        self.remain = self.rate_limit.limit;
        self.reset_datetime = now0();
    }
}

struct RestApiRateLimitsInner {
    /// 按权重计数的每分钟的限速(默认：每分钟6000权重值)
    weight: RestApiRateLimitInfo,
    /// 按请求次数计数的每10秒的下单次数限速(默认：每10秒50次下单请求)
    order_sec10: RestApiRateLimitInfo,
    /// 按请求次数计数的每天的下单次数限速(默认：每天16W次下单请求)
    order_day1: RestApiRateLimitInfo,
    /// 按请求次数计数的每5分钟的请求次数限速(默认：5分钟61000次请求)
    raw_requests: RestApiRateLimitInfo,
}

impl Default for RestApiRateLimitsInner {
    fn default() -> Self {
        let weight_rl = RateLimit {
            rate_limit_type: RateLimitType::RequestWeight,
            interval: RateLimitInterVal::Minute,
            interval_num: 1,
            limit: 6000,
            count: 0,
        };

        let order_sec10_rl = RateLimit {
            rate_limit_type: RateLimitType::Orders,
            interval: RateLimitInterVal::Second,
            interval_num: 10,
            limit: 50,
            count: 0,
        };

        let order_day1_rl = RateLimit {
            rate_limit_type: RateLimitType::Orders,
            interval: RateLimitInterVal::Day,
            interval_num: 1,
            limit: 160000,
            count: 0,
        };

        let raw_requests_rl = RateLimit {
            rate_limit_type: RateLimitType::RawRequests,
            interval: RateLimitInterVal::Minute,
            interval_num: 5,
            limit: 61000,
            count: 0,
        };

        Self {
            weight: RestApiRateLimitInfo::new(weight_rl),
            order_sec10: RestApiRateLimitInfo::new(order_sec10_rl),
            order_day1: RestApiRateLimitInfo::new(order_day1_rl),
            raw_requests: RestApiRateLimitInfo::new(raw_requests_rl),
        }
    }
}

#[derive(Clone)]
pub(crate) struct RestApiRateLimits {
    inner: Arc<RwLock<RestApiRateLimitsInner>>,
}

impl RestApiRateLimits {
    /// 获取到exchange_info信息之后，应立即调用 `update()` 方法进行更新
    pub async fn new() -> Self {
        let s = Self {
            inner: Arc::new(RwLock::new(RestApiRateLimitsInner::default())),
        };

        let ss = s.clone();
        tokio::spawn(async move {
            ss.run_tick().await;
        });

        s
    }

    /// 根据给定的exchange_info更新限速规则的最大值，而不是继续使用默认值
    pub async fn update(&self, exchange_info: &ExchangeInfo) {
        let limits = &exchange_info.rate_limits;
        let weight = limits
            .iter()
            .find(|x| matches!(x.rate_limit_type, RateLimitType::RequestWeight))
            .map(|x| x.limit);

        let order_sec10 = limits
            .iter()
            .find(|x| {
                matches!(x.rate_limit_type, RateLimitType::Orders)
                    && matches!(x.interval, RateLimitInterVal::Second)
            })
            .map(|x| x.limit);

        let order_day1 = limits
            .iter()
            .find(|x| {
                matches!(x.rate_limit_type, RateLimitType::Orders)
                    && matches!(x.interval, RateLimitInterVal::Day)
            })
            .map(|x| x.limit);

        let raw_requests = limits
            .iter()
            .find(|x| matches!(x.rate_limit_type, RateLimitType::RawRequests))
            .map(|x| x.limit);

        let mut inner = self.inner.write().await;
        if let Some(limit) = weight {
            inner.weight.rate_limit.limit = limit;
        }
        if let Some(limit) = order_sec10 {
            inner.order_sec10.rate_limit.limit = limit;
        }
        if let Some(limit) = order_day1 {
            inner.order_day1.rate_limit.limit = limit;
        }
        if let Some(limit) = raw_requests {
            inner.raw_requests.rate_limit.limit = limit;
        }
    }

    /// 尝试获取 n 个权重值(如果剩余权重值不够，将一直等待，直到权重值足够)
    pub async fn acquire_permits(&self, limit_param: RateLimitParam) {
        let (n, order_flag) = match limit_param {
            RateLimitParam::Weight(n) => (n, false),
            RateLimitParam::Order(n) => (n, true),
            RateLimitParam::Both(n) => (n, true),
        };

        loop {
            let mut inner = self.inner.write().await;

            if order_flag {
                if inner.weight.remain >= n
                    && inner.raw_requests.remain >= 1
                    && inner.order_sec10.remain >= 1
                    && inner.order_day1.remain >= 1
                {
                    inner.weight.remain -= n;
                    inner.raw_requests.remain -= 1;
                    inner.order_sec10.remain -= 1;
                    inner.order_day1.remain -= 1;
                    break;
                }
            } else if inner.weight.remain >= n && inner.raw_requests.remain >= 1 {
                inner.weight.remain -= n;
                inner.raw_requests.remain -= 1;
                break;
            }

            drop(inner);
            tokio::task::yield_now().await;
        }
    }

    /// 更新限速的值，传递的是当前限速时间段内已经使用的值
    ///
    /// date: 该Rest响应是在什么时间点发出的(格式"Fri, 25 Aug 2023 10:14:35 GMT")
    pub async fn set_permits(
        &self,
        response_date: String,
        weight_used: Option<u32>,
        order_sec10: Option<u32>,
        order_day1: Option<u32>,
    ) {
        let response_date = match response_date.to_dt_east0("%a, %d %b %Y %T %Z") {
            Ok(dt) => dt,
            Err(_) => {
                error!("parse str({}) to datetime", response_date);
                now0()
            }
        };

        let mut inner = self.inner.write().await;

        /*
         * 实际消耗的值(n)和计算消耗的值(max_limit - remain)，两者取max，
         * 因为请求响应后得到n，但响应前可能已经再次发起过请求，这时n值是滞后的。
         *
         * 但这样可能会出现一种问题：上一刻刚进行重置，而此处传递的是上一个限速时
         * 间段内的已使用的值(可能是一个很大的已使用值)，这将导致剩余值瞬间回到上
         * 一个时间段结束时的状态(很可能已经只剩下少量可使用值)，
         *
         * 例如，01:04:59.600的时候发送了一个请求，该时间段内的权重值只剩下5，
         * 该请求到了05分时才收到响应，该响应中的已使用值是5995，于是05分时进行更新，
         * 将瞬间从满血状态变成残血状态
         *
         * 因此，通过判断 response_date 的响应时间来判断该响应发生在重置前还是重置后，
         * 只有响应发生在重置后，才更新当前已经消耗的值
         */
        if inner.weight.reset_datetime < response_date {
            if let Some(n) = weight_used {
                let used = (inner.weight.rate_limit.limit - inner.weight.remain).max(n);
                inner.weight.remain = inner.weight.rate_limit.limit - used;
            }
        }

        if inner.order_sec10.reset_datetime < response_date {
            if let Some(n) = order_sec10 {
                let used = (inner.order_sec10.rate_limit.limit - inner.order_sec10.remain).max(n);
                inner.order_sec10.remain = inner.order_sec10.rate_limit.limit - used;
            }
        }

        if inner.order_day1.reset_datetime < response_date {
            if let Some(n) = order_day1 {
                let used = (inner.order_day1.rate_limit.limit - inner.order_day1.remain).max(n);
                inner.order_day1.remain = inner.order_day1.rate_limit.limit - used;
            }
        }
    }
}

impl RestApiRateLimits {
    /// tick，到了限速时间段的结束点就自动重置限速值
    pub async fn run_tick(&self) {
        let now = now0();
        let now_epoch = now.timestamp_millis();

        // 10秒间隔，每10秒tick一次
        let tick_intv = tokio::time::Duration::from_secs(10);

        // 下一个整10秒的epoch时间点，将从此开始进行tick
        let next_sec10_epoch = (now_epoch - now_epoch % 10_000) + 10_000;
        let start_tick = tokio::time::Instant::now()
            + tokio::time::Duration::from_millis((next_sec10_epoch - now_epoch) as u64);
        let mut ticker = tokio::time::interval_at(start_tick, tick_intv);

        loop {
            ticker.tick().await;

            let now = now0();
            let (h, m, s) = (now.hour(), now.minute(), now.second());

            if s % 10 == 0 {
                // 到了每日的整点
                if h == 0 && m == 0 && s == 0 {
                    self.reset_weight_permits().await;
                    self.reset_order_sec10_permits().await;
                    self.reset_order_day1_permits().await;
                    self.reset_raw_request_permits().await;
                    continue;
                }

                // 到了5分钟的整点
                if m % 5 == 0 && s == 0 {
                    self.reset_raw_request_permits().await;
                    continue;
                }

                // 到了每分钟的整点
                if s == 0 {
                    self.reset_weight_permits().await;
                    continue;
                }

                // 到了每10秒的整点
                self.reset_order_sec10_permits().await;
            }
        }
    }

    async fn reset_weight_permits(&self) {
        let mut x = self.inner.write().await;
        x.weight.reset_permits();
    }

    async fn reset_order_sec10_permits(&self) {
        let mut x = self.inner.write().await;
        x.order_sec10.reset_permits();
    }

    async fn reset_order_day1_permits(&self) {
        let mut x = self.inner.write().await;
        x.order_day1.reset_permits();
    }

    async fn reset_raw_request_permits(&self) {
        let mut x = self.inner.write().await;
        x.raw_requests.reset_permits();
    }
}

#[cfg(test)]
mod tt {
    use chrono_ext::ParseDateTimeExt;

    #[tokio::test]
    async fn t() {
        let str = "Fri, 25 Aug 2023 10:14:35 GMT";
        println!("{:?}", str.to_dt_east0("%a, %d %b %Y %T %Z"));

        // let now = now0();
        // let now_epoch = now.timestamp_millis();

        // // 10秒间隔
        // let tick_intv = tokio::time::Duration::from_secs(10);

        // // 下一个整10秒的epoch时间点，将从此开始进行tick
        // let next_sec10_epoch = (now_epoch - now_epoch % 10_000) + 10_000;
        // let start_tick = tokio::time::Instant::now()
        //     + tokio::time::Duration::from_millis((next_sec10_epoch - now_epoch) as u64);
        // let mut ticker = tokio::time::interval_at(start_tick, tick_intv);
        // ticker.tick().await;
        // println!("{}", now0());
    }
}
