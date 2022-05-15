#[derive(Default, Debug)]
pub struct Riot {
    timer: u8,
    clocks: usize,
    clocks_per_interval: usize,
}

impl Riot {
    pub fn set(&mut self, index: u16, value: u8) {
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

    pub fn get(&self, index: u16) -> u8 {
        self.timer
    }

    // TODO: This is not quite right. In reality it is definitely possible for the clock to advance
    // more that 256 cycles between updates. We need to handle that. Also once it counts down by 1
    // per cycle, it wraps and sets a negative flag.
    //
    // That flag is cleared when any timer is written OR read
    pub fn tick(&mut self, clocks: usize) {
        let total_clocks = self.clocks + clocks;
        if total_clocks >= self.clocks_per_interval {
            self.timer -= (total_clocks / self.clocks_per_interval) as u8;
            self.clocks = total_clocks % self.clocks_per_interval;
        } else {
            self.clocks += clocks;
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

        riot.tick(0x0100);
        assert_eq!(riot.get(0x0284), 0xA0);
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
