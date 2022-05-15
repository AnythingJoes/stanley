/// MemoryMap represents the memory layout of the 2600, including RIOT, TIA, RAM, and Program Memory
// TODO: make a const function to calculate these
const TIA_SIZE: usize = 64;
const RIOT_SIZE: usize = 0x029F - 0x0280;
const MEMORY_SIZE: usize = 0x00FF - 0x0080;
const PROGRAM_SIZE: usize = 0x2000 - 0x1000;

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
        let byte = self.mmap.program[(self.pc - 0x1000) as usize];
        self.pc += 1;
        byte
    }
}
