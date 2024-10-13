use std::time::{Duration, Instant};

pub struct Looper {
    update_start_time: Instant,
    loop_count: u32,
    delta_total: Duration,
    print_loop_time_rate: u32,
}

impl Default for Looper {
    fn default() -> Self {
        Self {
            update_start_time: Instant::now(),
            delta_total: Duration::ZERO,
            print_loop_time_rate: std::env::var("PRINT_LOOP_RATE")
                .ok()
                .and_then(|var| var.parse().ok())
                .unwrap_or(0),
            loop_count: 0,
        }
    }
}

impl Looper {
    pub const TARGET_LOOP_DELTA: Duration = Duration::from_millis(1000 / 60);

    pub fn start_loop(&mut self) {
        self.update_start_time = Instant::now();
    }

    pub async fn loop_end_wait(&mut self) {
        let loop_delta = self.update_start_time.elapsed();

        if self.print_loop_time_rate > 0 {
            self.delta_total += loop_delta;
            self.loop_count += 1;
            if self.loop_count % self.print_loop_time_rate == 0 {
                log::info!(
                    "Loop time: {:?}",
                    self.delta_total / self.print_loop_time_rate
                );
                self.delta_total = Duration::ZERO;
            }
        }

        if let Some(sleep_duration) = Self::TARGET_LOOP_DELTA.checked_sub(loop_delta) {
            tokio::time::sleep(sleep_duration).await;
        } else {
            log::warn!(
                "Main server loop took {:?} which is longer than target {:?}",
                loop_delta,
                Self::TARGET_LOOP_DELTA
            );
        }
    }
}
