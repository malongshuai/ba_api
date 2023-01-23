//! 实现接口限速规则，只实现`/api/*`接口的限频规则。
//!
//! 暂不考虑实现`/sapi/*`接口的限频规则。`/sapi/*`的每个接口都享有独立的限速权重，
//! 由于每个接口的权重值都较大(ip类限速是每分钟12000，UID类是每分钟180000)，理论上无需去限速  

use crate::now0;
use chrono::Timelike;
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, task, time};
use tracing::trace;

/// IP限频，除了下单操作，都采用IP限频规则
#[derive(Debug)]
struct IPRateLimit {
    /// 本阶段剩余的数量
    remain: usize,
}

impl IPRateLimit {
    /// 最大权重1200
    const MAX: usize = 1200;

    fn reset(&mut self) {
        self.remain = Self::MAX;
        trace!("IPRateLimit Reset: {:?}", self.remain);
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
        let left = Self::MAX - used;
        if left < self.remain {
            self.remain = left;
        }
        true
    }
}

impl Default for IPRateLimit {
    fn default() -> Self {
        Self { remain: Self::MAX }
    }
}

/// 秒级别的UID限速规则，每10秒最多只能下单50次
#[derive(Debug)]
struct UIDRateLimitSecLevel {
    /// 本阶段剩余的数量
    remain: usize,
}

impl Default for UIDRateLimitSecLevel {
    fn default() -> Self {
        Self { remain: Self::MAX }
    }
}

impl UIDRateLimitSecLevel {
    /// 最大权重50
    const MAX: usize = 50;

    fn reset(&mut self) {
        self.remain = Self::MAX;
        trace!("UIDRateLimitSecLevel Reset: {:?}", self.remain);
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
        let left = Self::MAX - used;
        if left < self.remain {
            self.remain = left;
        }
        true
    }
}

/// 日级别的UID限速规则，每天最多只能下单16W次
#[derive(Debug)]
struct UIDRateLimitDayLevel {
    /// 本阶段剩余的数量
    remain: usize,
}

impl UIDRateLimitDayLevel {
    /// 最大权重160000
    const MAX: usize = 160000;

    fn reset(&mut self) {
        self.remain = Self::MAX;
        trace!("UIDRateLimitDayLevel Reset: {:?}", self.remain);
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
        let left = Self::MAX - used;
        if left < self.remain {
            self.remain = left;
        }
        true
    }
}

impl Default for UIDRateLimitDayLevel {
    fn default() -> Self {
        Self { remain: Self::MAX }
    }
}

/// UID限速，下单时使用UID限速规则
#[derive(Debug, Default)]
struct UIDRateLimit {
    secs: UIDRateLimitSecLevel,
    day: UIDRateLimitDayLevel,
}

impl UIDRateLimit {
    fn reset_secs(&mut self) {
        self.secs.reset();
    }

    fn reset_day(&mut self) {
        self.day.reset();
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
#[derive(Debug, Clone, Default)]
pub struct APIRateLimit {
    /// IP限速
    ip_limit: Arc<RwLock<IPRateLimit>>,

    /// UID限速，下单时使用该限速接口
    uid_limit: Arc<RwLock<UIDRateLimit>>,
}

impl APIRateLimit {
    /// ip限速规则的最大权重，值为1200
    pub const IP_WEIGHT_MAX: usize = IPRateLimit::MAX;
    /// uid秒级限速规则的最大权重，值为50
    pub const UID_SEC_WEIGHT_MAX: usize = UIDRateLimitSecLevel::MAX;
    /// uid秒级限速规则的最大权重，值为160000
    pub const UID_DAY_WEIGHT_MAX: usize = UIDRateLimitDayLevel::MAX;

    /// 开始执行限速规则，需spawn一个单独的任务来执行
    pub async fn new() -> Self {
        let rate_limit = Self::default();

        let r = rate_limit.clone();
        tokio::spawn(async move {
            r.rate_maintainer().await;
        });

        rate_limit
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

    async fn reset_ip_limit(&self) {
        self.ip_limit.write().await.reset();
    }

    async fn reset_uid_secs_limit(&self) {
        self.uid_limit.write().await.reset_secs();
    }

    #[allow(dead_code)]
    async fn reset_uid_day_limit(&self) {
        self.uid_limit.write().await.reset_day();
    }

    // 只获取一次写锁，同时重置日级和秒级的UID限速值
    async fn reset_uid_limit(&self) {
        let mut uid_limit = self.uid_limit.write().await;
        uid_limit.reset_day();
        uid_limit.reset_secs();
    }

    /// ip限速规则的时间间隔，值为1分钟，  
    /// uid秒级限速规则的时间间隔，值为10秒，  
    /// uid秒级限速规则的时间间隔，值为一天，  
    /// 均从UTC的整点开始计时  
    async fn rate_maintainer(&self) {
        let tick_interval = Duration::from_secs(1);

        // 先对齐到秒的开头
        loop {
            let now = now0().nanosecond();
            // 在秒的前10毫秒
            if now <= 10_000_000 {
                break;
            }
            time::sleep(time::Duration::from_millis(1)).await;
        }

        let mut intv = time::interval(tick_interval);
        loop {
            let now = now0();
            let (h, m, s) = (now.hour(), now.minute(), now.second());

            // 整10秒时(延迟1秒)，如果是UTC 00:00:00，则同时重置秒级和日级的限速值(放在一起重置将只获取一次写锁)，否则只重置秒级
            if s % 10 == 1 {
                if h == 0 && m == 0 && s == 1 {
                    self.reset_uid_limit().await;
                } else {
                    self.reset_uid_secs_limit().await;
                }
            }

            if s == 1 {
                self.reset_ip_limit().await;
            }

            intv.tick().await;
        }
    }
}

// #[cfg(test)]
// mod test_rate_limit {
//     use super::*;
//     use tokio::time;

//     fn now() -> String {
//         now8().format("%FT%T.%3f").to_string()
//     }

//     #[tokio::test]
//     async fn test_ip_rate_limit() {
//         let api_limit = APIRateLimit::new().await;
//         loop {
//             time::sleep(time::Duration::from_millis(100)).await;
//             api_limit.get_ip_permit(50).await;
//             println!("{}, {}", now(), api_limit.ip_permits_remain().await);
//         }
//     }
// }
