use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitType {
    RequestWeight,
    Orders,
    RawRequests,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitInterVal {
    Second,
    Minute,
    Day,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: RateLimitType,
    pub interval: RateLimitInterVal,
    pub interval_num: u32,
    pub limit: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitInfo {
    pub rate_limit_type: RateLimitType,
    pub interval: RateLimitInterVal,
    pub interval_num: u32,
    pub limit: u32,
    pub count: u32,
}

pub mod api_rate_limit {
    use chrono::{Local, Timelike};
    use std::{sync::Arc, time::Duration};
    use tokio::{
        sync::RwLock,
        task,
        time::{self, Instant},
    };
    /// IP限频，除了下单操作，都采用IP限频规则
    #[derive(Debug)]
    struct IPRateLimit {
        /// 本阶段剩余的数量
        remain: usize,
        /// 本阶段从何时开始计时
        tick: Instant,
    }

    impl IPRateLimit {
        /// 最大权重1200
        const MAX: usize = 1200;
        /// 重置间隔60秒
        const PERIOD: Duration = Duration::from_secs(60);

        fn reset(&mut self) {
            let now = Instant::now();
            if now.duration_since(self.tick) > Self::PERIOD {
                self.tick = now;
                self.remain = Self::MAX;
            }
        }
        fn force_reset(&mut self) {
            self.tick = Instant::now();
            self.remain = Self::MAX;
        }

        fn get(&mut self, n: usize) -> bool {
            if self.remain < n {
                return false;
            }

            self.remain -= n;
            true
        }

        fn remain(&self) -> usize {
            self.remain
        }

        fn set(&mut self, used: usize) -> bool {
            if used > Self::MAX {
                return false;
            }
            self.remain = Self::MAX - used;
            true
        }
    }

    impl Default for IPRateLimit {
        fn default() -> Self {
            Self {
                remain: Self::MAX,
                tick: Instant::now(),
            }
        }
    }

    /// 秒级别的UID限速规则，每10秒最多只能下单50次
    #[derive(Debug)]
    struct UIDRateLimitSecLevel {
        /// 本阶段剩余的数量
        remain: usize,
        /// 本阶段从何时开始计时
        tick: Instant,
    }

    impl Default for UIDRateLimitSecLevel {
        fn default() -> Self {
            Self {
                remain: Self::MAX,
                tick: Instant::now(),
            }
        }
    }

    impl UIDRateLimitSecLevel {
        /// 最大权重50
        const MAX: usize = 50;
        /// 重置间隔10秒
        const PERIOD: Duration = Duration::from_secs(10);

        fn reset(&mut self) {
            let now = Instant::now();
            if now.duration_since(self.tick) > Self::PERIOD {
                self.tick = now;
                self.remain = Self::MAX;
            }
        }

        fn force_reset(&mut self) {
            self.tick = Instant::now();
            self.remain = Self::MAX;
        }

        fn get(&mut self, n: usize) -> bool {
            if self.remain < n {
                return false;
            }

            self.remain -= n;
            true
        }

        fn remain(&self) -> usize {
            self.remain
        }

        fn set(&mut self, used: usize) -> bool {
            if used > Self::MAX {
                return false;
            }
            self.remain = Self::MAX - used;
            true
        }
    }

    /// 日级别的UID限速规则，每天最多只能下单16W次
    #[derive(Debug)]
    struct UIDRateLimitDayLevel {
        /// 本阶段剩余的数量
        remain: usize,
        /// 本阶段从何时开始计时
        tick: Instant,
    }

    impl UIDRateLimitDayLevel {
        /// 最大权重160000
        const MAX: usize = 160000;
        /// 重置间隔为一天
        const PERIOD: Duration = Duration::from_secs(86400);

        fn reset(&mut self) {
            let now = Instant::now();
            if now.duration_since(self.tick) > Self::PERIOD {
                self.tick = now;
                self.remain = Self::MAX;
            }
        }

        fn force_reset(&mut self) {
            self.tick = Instant::now();
            self.remain = Self::MAX;
        }

        fn get(&mut self, n: usize) -> bool {
            if self.remain < n {
                return false;
            }

            self.remain -= n;
            true
        }

        fn remain(&self) -> usize {
            self.remain
        }

        fn set(&mut self, used: usize) -> bool {
            if used > Self::MAX {
                return false;
            }
            self.remain = Self::MAX - used;
            true
        }
    }

    impl Default for UIDRateLimitDayLevel {
        fn default() -> Self {
            Self {
                remain: Self::MAX,
                tick: Instant::now(),
            }
        }
    }

    /// UID限速，下单时使用UID限速规则
    #[derive(Debug, Default)]
    struct UIDRateLimit {
        secs: UIDRateLimitSecLevel,
        day: UIDRateLimitDayLevel,
    }

    impl UIDRateLimit {
        fn reset(&mut self) {
            self.secs.reset();
            self.day.reset();
        }

        fn force_reset(&mut self) {
            self.secs.force_reset();
            self.day.force_reset();
        }

        fn get(&mut self, n: usize) -> bool {
            if self.secs.remain() < n || self.day.remain() < n {
                return false;
            }

            self.secs.get(n);
            self.day.get(n);
            true
        }

        fn secs_remain(&self) -> usize {
            self.secs.remain()
        }
        fn day_remain(&self) -> usize {
            self.day.remain()
        }

        fn set_secs(&mut self, used: usize) -> bool {
            self.secs.set(used)
        }
        fn set_day(&mut self, used: usize) -> bool {
            self.day.set(used)
        }

        fn set(&mut self, sec_used: usize, day_used: usize) -> bool {
            if sec_used > UIDRateLimitSecLevel::MAX || day_used > UIDRateLimitDayLevel::MAX {
                return false;
            }
            self.set_secs(sec_used);
            self.set_day(day_used);
            true
        }
    }

    /// `/api/xxx`接口的限速规则
    #[derive(Debug, Default)]
    pub struct APIRateLimit {
        /// IP限速
        ip_limit: Arc<RwLock<IPRateLimit>>,

        /// UID限速，下单时使用该限速接口
        uid_limit: Arc<RwLock<UIDRateLimit>>,
    }

    impl APIRateLimit {
        /// ip限速规则的最大权重，值为1200
        pub const IP_WEIGHT_MAX: usize = IPRateLimit::MAX;
        /// ip限速规则的时间间隔，值为1分钟
        pub const IP_WEIGHT_PERIOD: Duration = IPRateLimit::PERIOD;
        /// uid秒级限速规则的最大权重，值为50
        pub const UID_SEC_WEIGHT_MAX: usize = UIDRateLimitSecLevel::MAX;
        /// uid秒级限速规则的时间间隔，值为10秒
        pub const UID_SEC_WEIGHT_PERIOD: Duration = UIDRateLimitSecLevel::PERIOD;
        /// uid秒级限速规则的最大权重，值为160000
        pub const UID_DAY_WEIGHT_MAX: usize = UIDRateLimitDayLevel::MAX;
        /// uid秒级限速规则的时间间隔，值为一天
        pub const UID_DAY_WEIGHT_PERIOD: Duration = UIDRateLimitDayLevel::PERIOD;

        /// 开始执行限速规则，需spawn一个单独的任务来执行
        pub async fn run() {
            Self::default().rate_maintainer().await;
        }

        /// 取n个IP限速的权重，权重不足将一直等待，直到权重值充足
        pub async fn get_ip_permit(&self, n: usize) {
            loop {
                let mut ip_limit = self.ip_limit.write().await;
                if ip_limit.get(n) {
                    break;
                }
                drop(ip_limit);
                task::yield_now().await;
            }
        }

        /// 尝试取n个IP限速的权重，立即返回而不会等待，取成功返回true，权重不足将返回false
        pub async fn try_get_ip_permit(&self, n: usize) -> bool {
            self.ip_limit.write().await.get(n)
        }

        /// 查看IP限速的剩余权重
        pub async fn ip_permits_remain(&self) -> usize {
            self.ip_limit.write().await.remain()
        }

        /// 更新IP限速的权重，需提供已使用权重的数量，如果提供的数量大于`IP_WEIGHT_MAX`的值，不做任何事并返回false
        pub async fn set_ip_permit(&self, used: usize) -> bool {
            self.ip_limit.write().await.set(used)
        }

        /// 取n个UID限速的权重，权重不足将一直等待，直到权重值充足
        pub async fn get_uid_permit(&self, n: usize) {
            loop {
                let mut uid_limit = self.uid_limit.write().await;
                if uid_limit.get(n) {
                    break;
                }
                drop(uid_limit);
                task::yield_now().await;
            }
        }

        /// 尝试取n个UID限速的权重，立即返回而不会等待，取成功返回true，权重不足将返回false
        pub async fn try_get_uid_permit(&self, n: usize) -> bool {
            self.uid_limit.write().await.get(n)
        }

        /// 查看UID限速的剩余权重，返回秒级别的剩余权重和天级别的剩余权重
        pub async fn uid_permits_remain(&self) -> (usize, usize) {
            let uid_limit = self.uid_limit.write().await;
            (uid_limit.secs_remain(), uid_limit.day_remain())
        }

        /// 更新UID限速的权重，需提供秒级和日级别已使用权重的数量，如果提供的秒级已使用的权重数量
        /// 大于`UID_SEC_WEIGHT_MAX`，或提供的日级已使用的权重数量大于`UID_DAY_WEIGHT_MAX`的值，不做任何事并返回false
        pub async fn set_uid_permit(&self, sec_used: usize, day_used: usize) -> bool {
            self.uid_limit.write().await.set(sec_used, day_used)
        }

        async fn force_reset(&self) {
            let mut ip_limit = self.ip_limit.write().await;
            let mut uid_limit = self.uid_limit.write().await;

            ip_limit.force_reset();
            uid_limit.force_reset();
        }

        async fn reset(&self) {
            let mut ip_limit = self.ip_limit.write().await;
            let mut uid_limit = self.uid_limit.write().await;

            ip_limit.reset();
            uid_limit.reset();
        }

        async fn rate_maintainer(&self) {
            // 先找到下一个整10秒点的时间点
            let mut chrono_now = Local::now();
            let chrono_duration = chrono::Duration::seconds(1);
            let next_ten = loop {
                chrono_now = chrono_now + chrono_duration;
                if chrono_now.second() % 10 == 0 {
                    break chrono_now;
                }
            };
            // 下一个整10秒点距离此时此刻的时长
            let first_tick_period = next_ten
                .signed_duration_since(Local::now())
                .to_std()
                .unwrap();

            // 第一次计时后，重置一次，从此刻开始，计时起点将按照10秒对齐
            time::interval(first_tick_period).tick().await;
            self.force_reset().await;

            let tick_interval = Duration::from_secs(10);
            let mut intv = time::interval(tick_interval);
            loop {
                intv.tick().await;
                self.reset().await;
            }
        }
    }
}
