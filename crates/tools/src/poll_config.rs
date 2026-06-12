use std::time::Duration;

/// Controls how often the worker polls the Stellar Horizon payments endpoint.
#[derive(Debug, Clone)]
pub struct PollConfig {
    /// Base interval between polls.
    pub interval: Duration,
    /// Maximum interval after back-off (used when API errors occur).
    pub max_interval: Duration,
    /// The default interval this config was created with (for reset).
    default_interval: Duration,
}

impl Default for PollConfig {
    fn default() -> Self {
        let interval = Duration::from_secs(10);
        Self {
            interval,
            max_interval: Duration::from_secs(120),
            default_interval: interval,
        }
    }
}

impl PollConfig {
    /// Returns a config suited for high-throughput environments (shorter interval).
    pub fn high_frequency() -> Self {
        let interval = Duration::from_secs(5);
        Self {
            interval,
            max_interval: Duration::from_secs(60),
            default_interval: interval,
        }
    }

    /// Returns a config suited for low-traffic / cost-sensitive deployments.
    pub fn low_frequency() -> Self {
        let interval = Duration::from_secs(30);
        Self {
            interval,
            max_interval: Duration::from_secs(300),
            default_interval: interval,
        }
    }

    /// Doubles the interval up to `max_interval` for exponential back-off on errors.
    pub fn back_off(&mut self) {
        self.interval = (self.interval * 2).min(self.max_interval);
    }

    /// Resets the interval back to its original configured default.
    pub fn reset(&mut self) {
        self.interval = self.default_interval;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_interval_is_10_seconds() {
        let config = PollConfig::default();
        assert_eq!(config.interval, Duration::from_secs(10));
    }

    #[test]
    fn high_frequency_interval_is_5_seconds() {
        let config = PollConfig::high_frequency();
        assert_eq!(config.interval, Duration::from_secs(5));
    }

    #[test]
    fn low_frequency_interval_is_30_seconds() {
        let config = PollConfig::low_frequency();
        assert_eq!(config.interval, Duration::from_secs(30));
    }

    #[test]
    fn back_off_doubles_interval() {
        let mut config = PollConfig::default();
        config.back_off();
        assert_eq!(config.interval, Duration::from_secs(20));
    }

    #[test]
    fn back_off_respects_max() {
        let mut config = PollConfig::default();
        for _ in 0..10 { config.back_off(); }
        assert_eq!(config.interval, config.max_interval);
    }

    #[test]
    fn reset_restores_original_interval() {
        let mut config = PollConfig::high_frequency();
        config.back_off();
        config.back_off();
        config.reset();
        assert_eq!(config.interval, Duration::from_secs(5));
    }
}
