//! In-memory stand-ins for web-backend's Redis keys: a TTL cache of provider answers and a
//! fixed-window counter per key (`geocoding-rate-limit:<ip>:<bucket>`). A restart empties
//! both, which only costs a few repeat provider calls.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The most answers kept; past it the oldest-expiring half goes first.
const CACHE_MAX: usize = 4_000;

#[derive(Debug)]
pub struct Cache<V> {
    ttl: Duration,
    items: Mutex<HashMap<String, (Instant, V)>>,
}

impl<V: Clone> Cache<V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            items: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<V> {
        let items = lock(&self.items);
        items
            .get(key)
            .filter(|(expires, _)| *expires > Instant::now())
            .map(|(_, v)| v.clone())
    }

    pub fn put(&self, key: String, value: V) {
        let mut items = lock(&self.items);
        let now = Instant::now();
        items.retain(|_, (expires, _)| *expires > now);
        if items.len() >= CACHE_MAX {
            let mut expiries: Vec<Instant> = items.values().map(|(e, _)| *e).collect();
            expiries.sort_unstable();
            let cut = expiries[expiries.len() / 2];
            items.retain(|_, (expires, _)| *expires > cut);
        }
        items.insert(key, (now + self.ttl, value));
    }
}

/// At most `max` hits per key in each `window`.
#[derive(Debug)]
pub struct WindowLimit {
    max: u32,
    window: Duration,
    counts: Mutex<HashMap<String, (Instant, u32)>>,
}

impl WindowLimit {
    pub fn new(max: u32, window: Duration) -> Self {
        Self {
            max,
            window: window.max(Duration::from_secs(1)),
            counts: Mutex::new(HashMap::new()),
        }
    }

    /// Count one hit for `key`; false once this window's allowance is used up.
    pub fn allow(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut counts = lock(&self.counts);
        counts.retain(|_, (start, _)| now.duration_since(*start) < self.window);
        let entry = counts.entry(key.to_string()).or_insert((now, 0));
        if entry.1 >= self.max {
            return false;
        }
        entry.1 += 1;
        true
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_expires() {
        let cache = Cache::new(Duration::from_millis(30));
        cache.put("a".into(), 1);
        assert_eq!(cache.get("a"), Some(1));
        std::thread::sleep(Duration::from_millis(40));
        assert_eq!(cache.get("a"), None);
    }

    #[test]
    fn limits_per_window() {
        let limit = WindowLimit::new(2, Duration::from_mins(1));
        assert!(limit.allow("ip") && limit.allow("ip"));
        assert!(!limit.allow("ip"));
        assert!(limit.allow("other"));
    }
}
