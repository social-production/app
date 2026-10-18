//! Per-IP daily signup limit. In memory: a restart resets it, which is acceptable because
//! proof-of-work and the New standing level carry most of the anti-abuse weight.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct SignupLimit {
    per_day: u32,
    counts: Mutex<HashMap<String, (u64, u32)>>,
}

impl SignupLimit {
    pub fn new(per_day: u32) -> Self {
        Self {
            per_day,
            counts: Mutex::new(HashMap::new()),
        }
    }

    /// Count one signup from `ip`; false once today's allowance is used up.
    pub fn allow(&self, ip: &str) -> bool {
        let today = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() / 86_400);
        let mut counts = self
            .counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        counts.retain(|_, (day, _)| *day == today);
        let entry = counts.entry(ip.to_string()).or_insert((today, 0));
        if entry.1 >= self.per_day {
            return false;
        }
        entry.1 += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::SignupLimit;

    #[test]
    fn limits_per_ip() {
        let limit = SignupLimit::new(2);
        assert!(limit.allow("1.2.3.4") && limit.allow("1.2.3.4"));
        assert!(!limit.allow("1.2.3.4"));
        assert!(limit.allow("5.6.7.8"));
    }
}
