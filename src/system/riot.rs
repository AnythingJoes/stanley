use std::fmt;

#[derive(Default, Debug)]
pub struct Riot {
    timer: u8,
    clocks: usize,
    clocks_per_interval: usize,
    timint: bool,
    pub timer_reset: bool,
}

impl Riot {
    // TODO: There are other things to set other than the timer. This will fail eventually
    pub fn set(&mut self, index: u16, value: u8) {
        self.timint = false;
        self.timer_reset = true;
        self.timer = value;
        self.clocks_per_interval = match index {
            0x14 => 1,
            0x15 => 8,
            0x16 => 64,
            0x17 => 1024,
            _ => todo!("RIOT write not implemented for {:X}", index),
        };
        // The time counts down on the next clock cycle
        self.clocks = self.clocks_per_interval - 1;
    }

    // TODO this only gets timer, there are other values here
    // timint is only reset if the timer is read
    pub fn get(&mut self, index: u16) -> u8 {
        if index & 0x0284 == 0x0284 {
            self.timint = false;
            return self.timer;
        }
        todo!("RIOT read not implemented for {:X}", index);
    }

    pub fn tick(&mut self, clocks: usize) {
        if self.clocks_per_interval == 0 || self.timer_reset {
            return;
        }

        let total_clocks = self.clocks + clocks;
        let timer_ticks = (total_clocks / self.clocks_per_interval) as u8;
        let (value, did_overflow) = self.timer.overflowing_sub(timer_ticks);

        if did_overflow {
            self.timer = 0xFF;
            let overflow_ticks = 0xFF - value;
            self.timer -= (overflow_ticks as usize * self.clocks_per_interval) as u8;
            self.timer -= (total_clocks % self.clocks_per_interval) as u8;
            self.clocks_per_interval = 1;
            self.timint = true;
        } else {
            self.timer = value;
            self.clocks = total_clocks % self.clocks_per_interval;
        }
    }
}

impl fmt::Display for Riot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "
RIOT\r\n
Timer: {:03}  | Timer Width  {:04} | TIMINT: {}\r\n\r\n
            ",
            self.timer, self.clocks_per_interval, self.timint
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_clock_timer() {
        let mut riot = Riot::default();
        riot.set(0x14, 100);
        riot.timer_reset = false;
        assert_eq!(riot.get(0x0284), 100);

        riot.tick(1);
        assert_eq!(riot.get(0x0284), 99);

        riot.tick(3);
        assert_eq!(riot.get(0x0284), 96);

        riot.tick(1024);
        assert_eq!(riot.get(0x0284), 96);
    }

    #[test]
    fn test_8_clock_timer() {
        let mut riot = Riot {
            timint: true,
            ..Default::default()
        };
        riot.set(0x15, 3);
        riot.timer_reset = false;
        assert!(!riot.timint);
        assert_eq!(riot.get(0x0284), 3);

        riot.tick(9);
        assert_eq!(riot.get(0x0284), 1);

        riot.tick(8);
        assert_eq!(riot.get(0x0284), 0);

        riot.tick(7);
        assert_eq!(riot.get(0x0284), 0);

        riot.tick(1);
        assert_eq!(riot.get(0x0284), 0xFF);

        riot.set(0x15, 5);
        riot.timer_reset = false;
        assert!(!riot.timint);
        riot.tick(42);

        assert!(riot.timint);
        assert_eq!(riot.get(0x0284), 0xFE);
    }

    #[test]
    fn test_64_clock_timer() {
        let mut riot = Riot::default();
        riot.set(0x16, 100);
        riot.timer_reset = false;
        assert_eq!(riot.get(0x0284), 100);

        riot.tick(64);
        assert_eq!(riot.get(0x0284), 99);

        riot.tick(1);
        assert_eq!(riot.get(0x0284), 98);

        riot.tick(66);
        assert_eq!(riot.get(0x0284), 97);

        riot.tick(128);
        assert_eq!(riot.get(0x0284), 95);
    }

    #[test]
    fn test_1024_timer() {
        let mut riot = Riot::default();
        riot.set(0x17, 100);
        riot.timer_reset = false;
        assert_eq!(riot.get(0x0284), 100);

        riot.tick(1024);
        assert_eq!(riot.get(0x0284), 99);

        riot.tick(1);
        assert_eq!(riot.get(0x0284), 98);
    }
}
