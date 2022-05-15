#[derive(Default, Debug)]
pub struct Riot {
    timer: u8,
    clocks: usize,
    clocks_per_interval: usize,
    timint: bool,
}

impl Riot {
    pub fn set(&mut self, index: u16, value: u8) {
        self.timint = false;
        self.timer = value;
        self.clocks = 0;
        self.clocks_per_interval = match index {
            0x14 => 1,
            0x15 => 8,
            0x16 => 64,
            0x17 => 1024,
            _ => return,
        };
    }

    // TODO this only gets timer, there are other values here
    // timint is only reset if the timer is read
    pub fn get(&mut self, index: u16) -> u8 {
        self.timint = false;
        self.timer
    }

    pub fn tick(&mut self, clocks: usize) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_clock_timer() {
        let mut riot = Riot::default();
        riot.set(0x14, 100);
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
        let mut riot = Riot::default();
        riot.timint = true;
        riot.set(0x15, 3);
        assert_eq!(riot.timint, false);
        assert_eq!(riot.get(0x0284), 3);

        riot.tick(8);
        assert_eq!(riot.get(0x0284), 2);

        riot.tick(16);
        assert_eq!(riot.get(0x0284), 0);

        riot.tick(7);
        assert_eq!(riot.get(0x0284), 0);

        riot.tick(1);
        assert_eq!(riot.get(0x0284), 0xFF);

        riot.set(0x15, 5);
        assert_eq!(riot.timint, false);
        riot.tick(49);

        assert_eq!(riot.timint, true);
        assert_eq!(riot.get(0x0284), 0xFE);
        assert_eq!(riot.timint, false)
    }

    #[test]
    fn test_64_clock_timer() {
        let mut riot = Riot::default();
        riot.set(0x16, 100);
        assert_eq!(riot.get(0x0284), 100);

        riot.tick(63);
        assert_eq!(riot.get(0x0284), 100);

        riot.tick(1);
        assert_eq!(riot.get(0x0284), 99);

        riot.tick(66);
        assert_eq!(riot.get(0x0284), 98);

        riot.tick(128);
        assert_eq!(riot.get(0x0284), 96);
    }

    #[test]
    fn test_1024_timer() {
        let mut riot = Riot::default();
        riot.set(0x17, 100);
        assert_eq!(riot.get(0x0284), 100);

        riot.tick(1023);
        assert_eq!(riot.get(0x0284), 100);

        riot.tick(1);
        assert_eq!(riot.get(0x0284), 99);
    }
}
