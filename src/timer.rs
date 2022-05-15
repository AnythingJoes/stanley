use std::time::Duration;

#[cfg(test)]
use fake_clock::FakeClock as Instant;
#[cfg(not(test))]
use std::time::Instant;

#[cfg(not(test))]
use spin_sleep::sleep;
#[cfg(test)]
fn sleep(durr: Duration) {
    use fake_clock::FakeClock;
    FakeClock::advance_time(durr.as_millis() as u64);
}

pub struct Timer {
    instant: Instant,
    render_instant: Instant,
    pub runover: Duration,
}

impl Timer {
    pub fn start() -> Self {
        Timer {
            instant: Instant::now(),
            render_instant: Instant::now(),
            runover: Duration::ZERO,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.instant.elapsed()
    }

    pub fn should_render(&self) -> bool {
        self.render_instant.elapsed() > Duration::from_millis(16)
    }

    pub fn did_render(&mut self) {
        self.render_instant = Instant::now();
    }

    // TODO: Fix this thing I didn't expect to happen where instructions seem to take longer than
    // they should
    pub fn pause_for(&mut self, dur: Duration) {
        let elapsed = self.elapsed();
        if dur < elapsed {
            self.runover += elapsed - dur;
        }
        let actual = dur.saturating_sub(elapsed);
        if self.runover < actual {
            let should_sleep = actual - self.runover;
            let now = Instant::now();

            sleep(should_sleep);
            self.runover = now.elapsed() - should_sleep;
        } else {
            self.runover -= actual;
        }

        self.instant = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_a_timer() {
        use fake_clock::FakeClock;
        let timer = Timer::start();
        let earlier = timer.elapsed();
        FakeClock::advance_time(10);
        assert!(timer.elapsed() > earlier);
    }

    #[test]
    // NOTE: Fake clock does not support times less than 1 millisecond these times are unrealistic
    // for our use case
    fn test_pause_for() {
        use fake_clock::FakeClock;
        let mut timer = Timer::start();
        let now = Instant::now();
        timer.pause_for(Duration::from_millis(83_700));
        FakeClock::advance_time(10);
        assert!(now.elapsed() > Duration::from_millis(83_700));
    }

    #[test]
    fn test_pause_for_too_long() {
        let mut timer = Timer::start();
        timer.runover = Duration::from_millis(20);
        timer.pause_for(Duration::from_millis(10));
    }
}
