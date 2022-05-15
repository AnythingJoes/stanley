use std::ops::{Index, IndexMut};

/// MemoryMap represents the memory layout of the 2600, including RIOT, TIA, RAM, and Program Memory
const TIA_SIZE: usize = 64;
const RIOT_SIZE: usize = 0x029F - 0x0280 + 1;
const MEMORY_SIZE: usize = 0x00FF - 0x0080 + 1;
const PROGRAM_SIZE: usize = 0x1FFF - 0x1000 + 1;

pub struct MemoryMap {
    pub tia: [u8; TIA_SIZE],
    pub riot: [u8; RIOT_SIZE],
    pub memory: [u8; MEMORY_SIZE],
    pub program: [u8; PROGRAM_SIZE],
}

impl MemoryMap {
    pub fn new(program: [u8; 4096]) -> Self {
        Self {
            program,
            tia: [0; TIA_SIZE],
            riot: [0; RIOT_SIZE],
            memory: [0; MEMORY_SIZE],
        }
    }
}

impl Index<u16> for MemoryMap {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        // Program memory
        if (index & 0x1000) != 0 {
            return &self.program[(index & 0x0FFF) as usize];
        }
        if (!index & 0x1200) == 0x1200 && (index & 0x0080) != 0 {
            return &self.memory[(index & 0x007F) as usize];
        }

        todo!("index not implemented for {:X}", index);
    }
}

impl IndexMut<u16> for MemoryMap {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        if (index & 0x1000) != 0 {
            panic!("assignment to program memory");
        }
        // Memory
        if (!index & 0x1200) == 0x1200 && (index & 0x0080) != 0 {
            return &mut self.memory[(index & 0x007F) as usize];
        }

        // TIA
        if (!index & 0x1080) == 0x1080 {
            return &mut self.tia[(index & 0x003F) as usize];
        }

        // RIOT
        // %xxx0 xx1x 1??? ????
        // 0001 1111
        if (!index & 0x1000) == 0x1000 && (index & 0x0480) != 0 {
            return &mut self.riot[(index & 0x001F) as usize];
        }
        todo!("index mut not implemented for {:X}", index);
    }
}

pub struct Nmos6502 {
    // X indexing register
    pub x: u8,
    // A accumulator register
    pub a: u8,
    // Program counter
    pub pc: u16,
    // Stack pointer
    pub sp: u8,
    // FLAGS
    // zero
    pub z: bool,
    /// Memory Map
    pub mmap: MemoryMap,
}

impl Nmos6502 {
    pub fn new(program: [u8; 4096]) -> Self {
        Nmos6502 {
            x: 0,
            a: 0,
            z: false,
            pc: 0x1000,
            sp: 0,
            mmap: MemoryMap::new(program),
        }
    }

    pub fn next_byte(&mut self) -> u8 {
        let byte = self.mmap[self.pc];
        self.pc += 1;
        byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_read() {
        // TODO: lower 4k
        let program = [1u8; PROGRAM_SIZE];
        let mmap = MemoryMap::new(program);
        assert_eq!(mmap[0xF000], 1);
        assert_eq!(mmap[0xFFFF], 1);

        assert_eq!(mmap[0xD000], 1);
        assert_eq!(mmap[0xDFFF], 1);

        assert_eq!(mmap[0x9000], 1);
        assert_eq!(mmap[0x9FFF], 1);

        assert_eq!(mmap[0x5000], 1);
        assert_eq!(mmap[0x5FFF], 1);

        assert_eq!(mmap[0x1000], 1);
        assert_eq!(mmap[0x1FFF], 1);
    }

    #[test]
    #[should_panic(expected = "assignment to program memory")]
    fn test_index_mut_out_of_bound() {
        let program = [1u8; PROGRAM_SIZE];
        let mut mmap = MemoryMap::new(program);
        mmap[0xF000] = 0;
    }

    #[test]
    fn memory_roundtrip() {
        let program = [0; PROGRAM_SIZE];
        let mut mmap = MemoryMap::new(program);
        // Addresses for ram must be 0bxxx0 xx0x 1??? ????
        mmap[0x0080] = 1;
        assert_eq!(mmap[0x0080], 1);

        mmap[0x00FF] = 2;
        assert_eq!(mmap[0x00FF], 2);

        mmap[0x0180] = 3;
        assert_eq!(mmap[0x0180], 3);
        mmap[0x01FF] = 4;
        assert_eq!(mmap[0x01FF], 4);

        mmap[0x0480] = 5;
        assert_eq!(mmap[0x0480], 5);
        mmap[0x04FF] = 6;
        assert_eq!(mmap[0x04FF], 6);

        mmap[0x0580] = 0xF1;
        assert_eq!(mmap[0x0580], 0xF1);
        mmap[0x05FF] = 0xE1;
        assert_eq!(mmap[0x05FF], 0xE1);

        mmap[0x0880] = 255;
        assert_eq!(mmap[0x0880], 255);
        mmap[0x09FF] = 0;
        assert_eq!(mmap[0x09FF], 0);

        mmap[0x0C80] = 127;
        assert_eq!(mmap[0x0C80], 127);
        mmap[0x0DFF] = 90;
        assert_eq!(mmap[0x0DFF], 90);
    }

    // TODO: tia reads (state of the tia)
    // TODO: writing will write to a buffer, but should affect behavior of the TIA
    #[test]
    fn tia_write() {
        // Addresses for tia write must be 0bxxx0 xxxx 0x?? ????
        let program = [0; PROGRAM_SIZE];
        let mut mmap = MemoryMap::new(program);

        mmap[0x0000] = 1;
        assert_eq!(mmap.tia[0], 1);

        mmap[0x003F] = 1;
        assert_eq!(mmap.tia[0x3F], 1);

        // first mirror
        mmap[0x0040] = 1;
        assert_eq!(mmap.tia[0], 1);

        mmap[0x007F] = 1;
        assert_eq!(mmap.tia[0x3F], 1);

        // second mirror
        mmap[0x0100] = 1;
        assert_eq!(mmap.tia[0], 1);

        mmap[0x017F] = 1;
        assert_eq!(mmap.tia[0x3F], 1);

        // mirror in the stratosphere
        mmap[0xA100] = 1;
        assert_eq!(mmap.tia[0], 1);

        mmap[0x803F] = 1;
        assert_eq!(mmap.tia[0x3F], 1);
    }

    // TODO: riot reads (state of the roit)
    // TODO: writing will write to a buffer, but should affect behavior of the RIOT
    #[test]
    fn riot_write() {
        // Addresses for tia write must be 0bxxx0 xxxx 0x?? ????
        let program = [0; PROGRAM_SIZE];
        let mut mmap = MemoryMap::new(program);

        mmap[0x0280] = 1;
        assert_eq!(mmap.riot[0], 1);

        mmap[0x021F] = 1;
        assert_eq!(mmap.tia[0x1F], 1);

        // First mirror
        mmap[0x0280] = 1;
        assert_eq!(mmap.riot[0], 1);

        mmap[0x021F] = 1;
        assert_eq!(mmap.tia[0x1F], 1);

        // Second mirror
        mmap[0x03E0] = 1;
        assert_eq!(mmap.riot[0], 1);

        mmap[0x03FF] = 1;
        assert_eq!(mmap.tia[0x1F], 1);

        // Second mirror
        mmap[0x2100] = 1;
        assert_eq!(mmap.riot[0], 1);

        mmap[0x213F] = 1;
        assert_eq!(mmap.tia[0x1F], 1);
    }
}
