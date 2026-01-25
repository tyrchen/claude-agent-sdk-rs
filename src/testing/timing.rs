//! Timing simulation with deterministic jitter

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Duration;

/// Timing simulator with deterministic jitter
pub struct TimingSimulator {
    rng: StdRng,
    speed_factor: f64,
    #[allow(dead_code)]
    use_paused_time: bool,
}

impl TimingSimulator {
    /// Create with specific seed for reproducibility
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            speed_factor: 1.0,
            use_paused_time: true,
        }
    }

    /// Create with instant timing (no delays)
    pub fn instant() -> Self {
        Self {
            rng: StdRng::seed_from_u64(0),
            speed_factor: 0.0,
            use_paused_time: true,
        }
    }

    /// Set speed factor (0.0 = instant, 1.0 = normal, 2.0 = 2x slower)
    pub fn with_speed_factor(mut self, factor: f64) -> Self {
        self.speed_factor = factor;
        self
    }

    /// Calculate delay with jitter
    pub fn delay(&mut self, base_ms: u64, jitter_ms: u64) -> Duration {
        let jitter = if jitter_ms > 0 {
            self.rng.gen_range(0..=jitter_ms)
        } else {
            0
        };

        let total_ms = (base_ms + jitter) as f64 * self.speed_factor;
        Duration::from_millis(total_ms as u64)
    }

    /// Apply delay asynchronously
    pub async fn apply_delay(&mut self, base_ms: u64, jitter_ms: u64) {
        let delay = self.delay(base_ms, jitter_ms);
        if delay > Duration::ZERO {
            tokio::time::sleep(delay).await;
        }
    }
}

/// Pre-defined timing profiles
pub mod timing_profiles {
    use super::super::scenario::TimingDefaults;

    /// Instant responses (for fast tests)
    pub fn instant() -> TimingDefaults {
        TimingDefaults {
            inter_message_delay_ms: 0,
            jitter_ms: 0,
            initial_response_delay_ms: 0,
        }
    }

    /// Fast but realistic
    pub fn fast() -> TimingDefaults {
        TimingDefaults {
            inter_message_delay_ms: 10,
            jitter_ms: 5,
            initial_response_delay_ms: 20,
        }
    }

    /// Simulates typical network latency
    pub fn realistic() -> TimingDefaults {
        TimingDefaults {
            inter_message_delay_ms: 50,
            jitter_ms: 30,
            initial_response_delay_ms: 150,
        }
    }

    /// Simulates slow network
    pub fn slow() -> TimingDefaults {
        TimingDefaults {
            inter_message_delay_ms: 200,
            jitter_ms: 100,
            initial_response_delay_ms: 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_simulator_deterministic() {
        let mut sim1 = TimingSimulator::new(42);
        let mut sim2 = TimingSimulator::new(42);

        for _ in 0..10 {
            let d1 = sim1.delay(100, 50);
            let d2 = sim2.delay(100, 50);
            assert_eq!(d1, d2, "Same seed should produce same delays");
        }
    }

    #[test]
    fn test_timing_simulator_instant() {
        let mut sim = TimingSimulator::instant();
        let delay = sim.delay(1000, 500);
        assert_eq!(delay, Duration::ZERO);
    }

    #[test]
    fn test_timing_simulator_speed_factor() {
        let mut sim = TimingSimulator::new(0).with_speed_factor(2.0);
        let delay = sim.delay(100, 0);
        assert_eq!(delay, Duration::from_millis(200));
    }
}
