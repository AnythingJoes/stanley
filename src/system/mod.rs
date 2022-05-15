use std::fmt;

pub mod instructions;
mod riot;

use instructions::Instruction;
use riot::Riot;

const MEMORY_SIZE: usize = 0x00FF - 0x0080 + 1;
const PROGRAM_SIZE: usize = 0x1FFF - 0x1000 + 1;

pub struct System {
    pub chip: Nmos6507,
    pub riot: Riot,
    pub tia: Tia,
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
            tia: Tia::default(),
            clocks: 0,
            memory: [0; MEMORY_SIZE],
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
            return self.tia.set(index & 0x003F, value);
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
        if (!index & 0x1080) == 0x1080 {
            return self.tia.get(index & 0x000F);
        }

        if (!index & 0x1000) == 0x1000 && (index & 0x0480) != 0 {
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
        self.riot.tick(clocks);
        self.tia.tick(clocks);
    }

    pub fn execute(&mut self, inst: impl Instruction) {
        let ticks = inst.execute(self);
        self.tick(ticks);
        if self.tia.wsync {
            self.tick(self.tia.wsync_ticks());
            self.tia.wsync = false;
        }
    }
}

impl fmt::Display for System {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "
CLOCKS: {}\r\n
\r\n
RAM\r\n",
            self.clocks,
        )?;

        for i in 0..8 {
            for j in 0..16 {
                let memory = self.memory[i * 16 + j];
                write!(f, "{:02X} ", memory)?;
            }
            write!(f, "\r\n")?;
        }
        Ok(())
    }
}

const COLOR_CLOCKS_PER_LINE: usize = 228;
const SCAN_LINES: usize = 262;
const COLOR_CLOCKS_PER_FRAME: usize = COLOR_CLOCKS_PER_LINE * SCAN_LINES;
#[derive(Debug, Default)]
pub struct Tia {
    vsync: bool,
    vblank: bool,
    wsync: bool,
    // colors
    colupf: u8,
    colubk: u8,

    //ctrlpf
    pf_reflected: bool,

    //pf registers
    pf0: u8,
    pf1: u8,
    pf2: u8,

    // color clocks this frame
    color_clocks: usize,
}

impl Tia {
    fn set(&mut self, index: u16, value: u8) {
        match index {
            0x00 => self.vsync = (value & 0x02) != 0,
            // TODO: vblank does other thing on D6 and D7 pins, will need to be implemented
            0x01 => self.vblank = (value & 0x02) != 0,
            0x02 => self.wsync = true,
            // TODO: RSYNC: can be ignored in most cases. There is one game that depends on this being
            // handled correctly
            0x03 => (),
            0x04..=0x07 => (), // Ignored for now
            0x08 => self.colupf = value,
            0x09 => self.colubk = value,
            // TODO: other parts of ctrlpf
            0x0A => self.pf_reflected = (value & 0x01) == 1,
            0x0B..=0x0C => (), // Ignored for now
            0x0D => self.pf0 = value & 0xF0,
            0x0E => self.pf1 = value,
            0x0F => self.pf2 = value,
            0x10..=0x2C => (), // Ignored for now
            0x2D..=0x3F => (), // Unused
            _ => unreachable!("Tia set not implemented for {:04X} index", index),
        }
    }

    fn get(&self, index: u16) -> u8 {
        // TODO: Needs a real implementation
        // If it ends in 0xC, it's trying to read player 0 input in this case 0
        // is pressed and 1 in the sign bit is the default state. We want to
        // return the default state until we implement input
        if (index & 0x000F) == 0xC {
            return 0b1000_0000;
        }
        unimplemented!("Tia get not implemented for {:04X} index", index);
    }

    fn tick(&mut self, clocks: usize) {
        self.color_clocks = (self.color_clocks + clocks * 3) % COLOR_CLOCKS_PER_FRAME;
    }

    pub fn wsync_ticks(&self) -> usize {
        (COLOR_CLOCKS_PER_LINE - self.color_clocks % COLOR_CLOCKS_PER_LINE) / 3
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

impl fmt::Display for Nmos6507 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "
NMOS 6507\r\n
Registers: X({:02X}) Y({:02X}) A({:02X})   | PC: {:02X}   | SP: {:02X}   |\r\n\r\n
Flags: Z({}) N({}) C({})\r\n
            ",
            self.x, self.y, self.a, self.pc, self.sp, self.z, self.n, self.c
        )
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
