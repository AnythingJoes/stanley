pub mod instructions;
mod riot;

use instructions::Instruction;
use riot::Riot;

const MEMORY_SIZE: usize = 0x00FF - 0x0080 + 1;
const PROGRAM_SIZE: usize = 0x1FFF - 0x1000 + 1;

pub struct System {
    pub chip: Nmos6507,
    pub riot: Riot,
    pub memory: [u8; MEMORY_SIZE],
    pub program: [u8; PROGRAM_SIZE],
    // TODO temporarily track clocks
    pub clocks: usize,
}

impl System {
    pub fn new(program: [u8; 4096]) -> Self {
        Self {
            chip: Nmos6507::new(),
            riot: Riot::default(),
            clocks: 0,
            memory: [3; MEMORY_SIZE],
            program,
        }
    }

    pub fn memory_set(&mut self, index: u16, value: u8) {
        if (index & 0x1000) != 0 {
            panic!("assignment to program memory");
        }

        // Memory
        if (!index & 0x1200) == 0x1200 && (index & 0x0080) != 0 {
            return self.memory[(index & 0x007F) as usize] = value;
        }

        // // TIA
        if (!index & 0x1080) == 0x1080 {
            //     return &mut self.tia[(index & 0x003F) as usize];
            return;
        }

        // RIOT
        // 0b0000_0010_1001_0100
        // 0b0000_0010_1001_0100
        if (!index & 0x1000) == 0x1000 && (index & 0x0294) != 0 {
            return self.riot.set(index & 0x001F, value);
        }
        todo!("set not implemented for {:04X}", index);
    }

    pub fn memory_get(&mut self, index: u16) -> u8 {
        // Program memory
        if (index & 0x1000) != 0 {
            return self.program[(index & 0x0FFF) as usize];
        }

        // Memory
        if (!index & 0x1200) == 0x1200 && (index & 0x0080) != 0 {
            return self.memory[(index & 0x007F) as usize];
        }

        // TIA Read
        // TODO: Needs a real implementation
        // If it ends in 0xC, it's trying to read player 0 input in this case 0
        // is pressed and 1 in the sign bit is the default state. We want to
        // return the default state until we implement input
        if (!index & 0x1080) == 0x1080 {
            if (index & 0x000F) == 0xC {
                return 0b1000_0000;
            }
            return 0;
        }

        // RIOT Read
        if (!index & 0x1000) == 0x1000 && (index & 0x0480) != 0 {
            // INTIM timer check
            // if index & 0x0284 == 0x0284 { return &1; // anything but zero
            // }
            return self.riot.get(index);
        }

        todo!("index not implemented for {:04X}", index);
    }

    pub fn next_byte(&mut self) -> u8 {
        let byte = self.memory_get(self.chip.pc);
        self.chip.pc += 1;
        byte
    }

    pub fn tick(&mut self, clocks: usize) {
        self.clocks += clocks;
        self.riot.tick(clocks)
    }

    pub fn execute(&mut self, inst: impl Instruction) {
        let ticks = inst.execute(self);
        self.tick(ticks)
    }
}

#[derive(Default)]
pub struct Nmos6507 {
    /// X indexing register
    pub x: u8,
    /// Y indexing register
    pub y: u8,
    /// A accumulator register
    pub a: u8,
    /// Program counter
    pub pc: u16,
    /// Stack pointer
    pub sp: u8,
    /// FLAGS
    /// negative
    pub n: bool,
    /// zero
    pub z: bool,
    /// cary
    pub c: bool,
}

impl Nmos6507 {
    pub fn new() -> Self {
        Nmos6507 {
            pc: 0x1000,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_read() {
        let program = [1u8; PROGRAM_SIZE];
        let mut system = System::new(program);
        assert_eq!(system.memory_get(0xF000), 1);
        assert_eq!(system.memory_get(0xFFFF), 1);

        assert_eq!(system.memory_get(0xD000), 1);
        assert_eq!(system.memory_get(0xDFFF), 1);

        assert_eq!(system.memory_get(0x9000), 1);
        assert_eq!(system.memory_get(0x9FFF), 1);

        assert_eq!(system.memory_get(0x5000), 1);
        assert_eq!(system.memory_get(0x5FFF), 1);

        assert_eq!(system.memory_get(0x1000), 1);
        assert_eq!(system.memory_get(0x1FFF), 1);
    }

    #[test]
    #[should_panic(expected = "assignment to program memory")]
    fn test_set_out_of_bound() {
        let program = [1u8; PROGRAM_SIZE];
        let mut system = System::new(program);
        system.memory_set(0xF000, 0);
    }

    #[test]
    fn memory_roundtrip() {
        let program = [0; PROGRAM_SIZE];
        let mut system = System::new(program);
        // Addresses for ram must be 0bxxx0 xx0x 1??? ????
        system.memory_set(0x0080, 1);
        assert_eq!(system.memory_get(0x0080), 1);

        system.memory_set(0x00FF, 2);
        assert_eq!(system.memory_get(0x00FF), 2);

        system.memory_set(0x0180, 3);
        assert_eq!(system.memory_get(0x0180), 3);
        system.memory_set(0x01FF, 4);
        assert_eq!(system.memory_get(0x01FF), 4);

        system.memory_set(0x0480, 5);
        assert_eq!(system.memory_get(0x0480), 5);
        system.memory_set(0x04FF, 6);
        assert_eq!(system.memory_get(0x04FF), 6);

        system.memory_set(0x0580, 0xF1);
        assert_eq!(system.memory_get(0x0580), 0xF1);
        system.memory_set(0x05FF, 0xE1);
        assert_eq!(system.memory_get(0x05FF), 0xE1);

        system.memory_set(0x0880, 255);
        assert_eq!(system.memory_get(0x0880), 255);
        system.memory_set(0x09FF, 0);
        assert_eq!(system.memory_get(0x09FF), 0);

        system.memory_set(0x0C80, 127);
        assert_eq!(system.memory_get(0x0C80), 127);
        system.memory_set(0x0DFF, 90);
        assert_eq!(system.memory_get(0x0DFF), 90);
    }
}
