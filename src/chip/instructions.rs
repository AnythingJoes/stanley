use super::Nmos6502;

pub trait Instruction {
    const CODE: u8;
    fn execute(chip: &mut Nmos6502);
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
    fn execute(chip: &mut Nmos6502) {
        let arg = chip.next_byte();
        chip.z = arg == 0;
        chip.x = arg;
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
    fn execute(chip: &mut Nmos6502) {
        let arg = chip.next_byte();
        chip.z = arg == 0;
        chip.a = arg;
    }
}

// LDA
// FLAGS: N Z
// Mode: Zero Page
// Syntax: LDA #$44
// Hex: $A5
// Width: 2
// Timing: 2
pub struct LdaZ;
impl Instruction for LdaZ {
    const CODE: u8 = 0xA5;
    fn execute(chip: &mut Nmos6502) {
        let arg = chip.next_byte() as u16;
        let value = chip.mmap[arg];
        chip.z = value == 0;
        chip.a = value;
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
    fn execute(chip: &mut Nmos6502) {
        let arg = chip.next_byte();
        chip.mmap[arg as u16] = chip.a;
    }
}

// STA
// FLAGS: None
// Mode: Zero Page,X
// Syntax: STA $44
// Hex: $95
// Width: 2
// Timing: 4
pub struct StaZX;
impl Instruction for StaZX {
    const CODE: u8 = 0x95;
    fn execute(chip: &mut Nmos6502) {
        let arg = chip.next_byte();
        let index = (arg + chip.x) as u16;
        chip.mmap[index] = chip.a;
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
    fn execute(chip: &mut Nmos6502) {
        let low = chip.next_byte() as u16;
        let high = chip.next_byte() as u16;
        let addr = (high << 8) + low;
        chip.mmap[addr] = chip.x;
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
    fn execute(chip: &mut Nmos6502) {
        chip.x = chip.x.wrapping_add(1);
        chip.z = chip.x == 0;
    }
}

// Branching Instructions
//
// BNE
// FLAGS:
// Syntax: BNE Label
// Hex: $D0
// Width: 1
// Timing: 2, +1 if taken, +1 if across page boundary
pub struct Bne;
impl Instruction for Bne {
    const CODE: u8 = 0xD0;
    fn execute(chip: &mut Nmos6502) {
        let arg = chip.next_byte() as i8;
        if !chip.z {
            chip.pc = chip.pc.wrapping_add(arg as u16);
        }
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
    fn execute(chip: &mut Nmos6502) {
        chip.sp = chip.x;
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
    fn execute(chip: &mut Nmos6502) {
        let low = chip.next_byte() as u16;
        let high = chip.next_byte() as u16;
        let jump_address = (high << 8) + low;

        let ret_low = chip.pc as u8;
        let ret_high = (chip.pc >> 8) as u8;
        chip.mmap[chip.sp as u16] = ret_high;
        chip.sp -= 1;
        chip.mmap[chip.sp as u16] = ret_low - 1;
        chip.sp -= 1;
        chip.pc = jump_address;
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
    fn execute(chip: &mut Nmos6502) {
        chip.sp += 1;
        let low = chip.mmap[chip.sp as u16] as u16;
        chip.sp += 1;
        let high = chip.mmap[chip.sp as u16] as u16;
        chip.pc = (high << 8) + low + 1;
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
    fn execute(chip: &mut Nmos6502) {
        let arg = chip.next_byte();
        chip.a ^= arg;
    }
}
