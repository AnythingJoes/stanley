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

        // Memory
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
        if (!index & 0x1000) == 0x1000 && (index & 0x0480) != 0 {
            return &mut self.riot[(index & 0x001F) as usize];
        }
        todo!("index mut not implemented for {:X}", index);
    }
}

