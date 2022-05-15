pub mod instructions;
mod memory_map;

use memory_map::MemoryMap;

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
