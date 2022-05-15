use super::System;

pub trait Instruction {
    const CODE: u8;
    /// executes the instruction and then returns the number of clocks that should be counted.
    ///
    /// This is the first step toward emulating how these instructions are actually run. In this
    /// case, we need to make sure reads and writes happen before the last tick. This strategy is
    /// an attempt to avoid emulating the data and address buss directly. This may need to be
    /// changed in the future
    fn execute(&self, system: &mut System) -> usize;
}

// LDX
// FLAGS: N Z
// Mode: Immediate
// Syntax: LDX #$44
// Hex: $A2
// Width: 2
// Timing: 2
pub struct LdxI;
impl Instruction for LdxI {
    const CODE: u8 = 0xA2;
    fn execute(&self, system: &mut System) -> usize {
        let arg = system.next_byte();
        system.chip.z = arg == 0;
        system.chip.n = (arg as i8) < 0;

        system.chip.x = arg;
        2
    }
}

// LDA
// FLAGS: N Z
// Mode: Immediate
// Syntax: LDA #$44
// Hex: $A9
// Width: 2
// Timing: 2
pub struct LdaI;
impl Instruction for LdaI {
    const CODE: u8 = 0xA9;
    fn execute(&self, system: &mut System) -> usize {
        let arg = system.next_byte();
        system.chip.z = arg == 0;
        system.chip.n = (arg as i8) < 0;

        system.chip.a = arg;
        2
    }
}

// LDA
// FLAGS: N Z
// Mode: Zero Page
// Syntax: LDA $44
// Hex: $A5
// Width: 2
// Timing: 2
pub struct LdaZ;
impl Instruction for LdaZ {
    const CODE: u8 = 0xA5;
    fn execute(&self, system: &mut System) -> usize {
        let arg = system.next_byte() as u16;
        let value = system.memory_get(arg);
        system.chip.z = value == 0;
        system.chip.n = (value as i8) < 0;

        system.chip.a = value;
        2
    }
}

// LDA
// FLAGS: N Z
// Mode: Absolute
// Syntax: LDA $4400
// Hex: $AD
// Width: 3
// Timing: 4
pub struct LdaA;
impl Instruction for LdaA {
    const CODE: u8 = 0xAD;
    fn execute(&self, system: &mut System) -> usize {
        let low = system.next_byte() as u16;
        let high = system.next_byte() as u16;
        let value = system.memory_get((high << 8) + low);
        system.chip.z = value == 0;
        system.chip.n = (value as i8) < 0;

        system.chip.a = value;
        4
    }
}

// LDA
// FLAGS: N Z
// Mode: Absolute,Y
// Syntax: LDA $4400,Y
// Hex: $B9
// Width: 3
// Timing: 4, +1 if page boundary crossed
pub struct LdaAY;
impl Instruction for LdaAY {
    const CODE: u8 = 0xB9;
    fn execute(&self, system: &mut System) -> usize {
        let low = system.next_byte() as u16;
        let high = system.next_byte() as u16;
        let addr = (high << 8) + low;
        let value = system.memory_get(addr + (system.chip.y as u16));
        system.chip.z = value == 0;
        system.chip.n = (value as i8) < 0;

        system.chip.a = value;
        4
    }
}

// STA
// FLAGS: None
// Mode: Zero Page
// Syntax: STA $44
// Hex: $85
// Width: 2
// Timing: 3
pub struct StaZ;
impl Instruction for StaZ {
    const CODE: u8 = 0x85;
    fn execute(&self, system: &mut System) -> usize {
        let arg = system.next_byte();
        system.memory_set(arg as u16, system.chip.a);
        3
    }
}

// STA
// FLAGS: None
// Mode: Zero Page,X
// Syntax: STA $44,X
// Hex: $95
// Width: 2
// Timing: 4
pub struct StaZX;
impl Instruction for StaZX {
    const CODE: u8 = 0x95;
    fn execute(&self, system: &mut System) -> usize {
        let arg = system.next_byte();
        let index = (arg + system.chip.x) as u16;
        system.memory_set(index, system.chip.a);
        4
    }
}

// STA
// FLAGS: None
// Mode: Zero Page,X
// Syntax: STA $44,X
// Hex: $8D
// Width: 3
// Timing: 4
pub struct StaA;
impl Instruction for StaA {
    const CODE: u8 = 0x8D;
    fn execute(&self, system: &mut System) -> usize {
        let low = system.next_byte() as u16;
        let high = system.next_byte() as u16;
        let index = (high << 8) + low;
        system.memory_set(index, system.chip.a);
        4
    }
}

// STX
// FLAGS: None
// Mode: Absolute
// Syntax: STX $4444
// Hex: $8E
// Width: 3
// Timing: 4
pub struct StxA;
impl Instruction for StxA {
    const CODE: u8 = 0x8E;
    fn execute(&self, system: &mut System) -> usize {
        let low = system.next_byte() as u16;
        let high = system.next_byte() as u16;
        let addr = (high << 8) + low;
        system.memory_set(addr, system.chip.x);
        4
    }
}

// INX
// FLAGS: N Z
// Syntax: INX
// Hex: $E8
// Width: 1
// Timing: 2
pub struct Inx;
impl Instruction for Inx {
    const CODE: u8 = 0xE8;
    fn execute(&self, system: &mut System) -> usize {
        system.chip.x = system.chip.x.wrapping_add(1);

        system.chip.z = system.chip.x == 0;
        system.chip.n = (system.chip.x as i8) < 0;
        2
    }
}

// DEX
// FLAGS: N Z
// Syntax: DEX
// Hex: $CA
// Width: 1
// Timing: 2
pub struct Dex;
impl Instruction for Dex {
    const CODE: u8 = 0xCA;
    fn execute(&self, system: &mut System) -> usize {
        system.chip.x = system.chip.x.wrapping_sub(1);

        system.chip.z = system.chip.x == 0;
        system.chip.n = (system.chip.x as i8) < 0;
        2
    }
}

// Branching Instructions
//
// BNE
// FLAGS:
// Syntax: BNE Label
// Hex: $D0
// Width: 2
// Timing: 2, +1 if taken, +1 if across page boundary
pub struct Bne;
impl Instruction for Bne {
    const CODE: u8 = 0xD0;
    fn execute(&self, system: &mut System) -> usize {
        let arg = system.next_byte() as i8;
        let mut ticks = 2;
        if !system.chip.z {
            ticks += 1;
            let addr = system.chip.pc.wrapping_add(arg as u16);
            if (addr & 0xFF00) != (system.chip.pc & 0xFF00) {
                ticks += 1;
            }
            system.chip.pc = addr;
        }
        ticks
    }
}

// BMI
// FLAGS:
// Syntax: BNE Label
// Hex: $30
// Width: 2
// Timing: 2, +1 if taken, +1 if across page boundary
pub struct Bmi;
impl Instruction for Bmi {
    const CODE: u8 = 0x30;
    fn execute(&self, system: &mut System) -> usize {
        let arg = system.next_byte() as i8;
        let mut ticks = 2;
        if system.chip.n {
            ticks += 1;
            let addr = system.chip.pc.wrapping_add(arg as u16);
            if (addr & 0xFF00) != (system.chip.pc & 0xFF00) {
                ticks += 1;
            }
            system.chip.pc = addr;
        }
        ticks
    }
}

// JMP
// FLAGS:
// Syntax: JMP Label
// Hex: $4C
// Width: 3
// Timing: 3
pub struct Jmp;
impl Instruction for Jmp {
    const CODE: u8 = 0x4C;
    fn execute(&self, system: &mut System) -> usize {
        let low = system.next_byte() as u16;
        let high = system.next_byte() as u16;
        system.chip.pc = (high << 8) + low;
        3
    }
}

// Stack Instructions
//
// TXS
// FLAGS:
// Syntax: TXS
// Hex: $9A
// Width: 1
// Timing: 2
pub struct Txs;
impl Instruction for Txs {
    const CODE: u8 = 0x9A;
    fn execute(&self, system: &mut System) -> usize {
        system.chip.sp = system.chip.x;
        2
    }
}

// Register Instructions
//
// TXA
// FLAGS: N Z
// Syntax: TXA
// Hex: $8A
// Width: 1
// Timing: 2
pub struct Txa;
impl Instruction for Txa {
    const CODE: u8 = 0x8A;
    fn execute(&self, system: &mut System) -> usize {
        system.chip.a = system.chip.x;
        system.chip.z = system.chip.a == 0;
        system.chip.n = (system.chip.a as i8) < 0;
        2
    }
}

// TAY
// FLAGS: N Z
// Syntax: TAY
// Hex: $A8
// Width: 1
// Timing: 2
pub struct Tay;
impl Instruction for Tay {
    const CODE: u8 = 0xA8;
    fn execute(&self, system: &mut System) -> usize {
        system.chip.y = system.chip.a;
        system.chip.z = system.chip.y == 0;
        system.chip.n = (system.chip.y as i8) < 0;
        2
    }
}

// Subroutine Instructions
//
// JSR
// FLAGS:
// Syntax: JSR Label
// Hex: $20
// Width: 3
// Timing: 6
pub struct Jsr;
impl Instruction for Jsr {
    const CODE: u8 = 0x20;
    fn execute(&self, system: &mut System) -> usize {
        let low = system.next_byte() as u16;
        let high = system.next_byte() as u16;
        let jump_address = (high << 8) + low;

        let ret_low = system.chip.pc as u8;
        let ret_high = (system.chip.pc >> 8) as u8;
        system.memory_set(system.chip.sp as u16, ret_high);
        system.chip.sp -= 1;
        system.memory_set(system.chip.sp as u16, ret_low - 1);
        system.chip.sp -= 1;
        system.chip.pc = jump_address;
        6
    }
}

// RTS
// FLAGS:
// Syntax: RTS
// Hex: $60
// Width: 1
// Timing: 6
pub struct Rts;
impl Instruction for Rts {
    const CODE: u8 = 0x60;
    fn execute(&self, system: &mut System) -> usize {
        system.chip.sp += 1;
        let low = system.memory_get(system.chip.sp as u16) as u16;
        system.chip.sp += 1;
        let high = system.memory_get(system.chip.sp as u16) as u16;
        system.chip.pc = (high << 8) + low + 1;
        6
    }
}

// Bitwise Operations

// EOR
// FLAGS: N Z
// Syntax: EOR #$44
// Mode: Immediate
// Hex: $49
// Width: 2
// Timing: 2
pub struct Eor;
impl Instruction for Eor {
    const CODE: u8 = 0x49;
    fn execute(&self, system: &mut System) -> usize {
        let arg = system.next_byte();
        system.chip.n = (arg as i8) < 0;
        system.chip.z = arg == 0;
        system.chip.a ^= arg;
        2
    }
}

// LSR
// FLAGS: N Z C
// Syntax: LSR (A)
// Mode: Accumulator
// Hex: $4A
// Width: 1
// Timing: 2
pub struct Lsr;
impl Instruction for Lsr {
    const CODE: u8 = 0x4A;
    fn execute(&self, system: &mut System) -> usize {
        let arg = system.next_byte();
        system.chip.c = system.chip.a & 0x01 == 0;
        system.chip.a >>= 1;
        system.chip.a ^= arg;
        2
    }
}

// Other codes
//
// NOP
// FLAGS:
// Syntax: NOP
// Mode: Implied
// Hex: $EA
// Width: 1
// Timing: 2
pub struct Nop;
impl Instruction for Nop {
    const CODE: u8 = 0xEA;
    fn execute(&self, _system: &mut System) -> usize {
        2
    }
}