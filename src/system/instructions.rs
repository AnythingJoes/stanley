use std::collections::HashMap;
use std::fmt;

use super::System;

// LDX
// FLAGS: N Z
// Mode: Immediate
// Syntax: LDX #$44
// Hex: $A2
// Width: 2
// Timing: 2
fn execute_ldx_i(system: &mut System) -> usize {
    let arg = system.next_byte();
    system.chip.z = arg == 0;
    system.chip.n = (arg as i8) < 0;

    system.chip.x = arg;
    2
}

// LDA
// FLAGS: N Z
// Mode: Immediate
// Syntax: LDA #$44
// Hex: $A9
// Width: 2
// Timing: 2
fn execute_lda_i(system: &mut System) -> usize {
    let arg = system.next_byte();
    system.chip.z = arg == 0;
    system.chip.n = (arg as i8) < 0;

    system.chip.a = arg;
    2
}

// LDA
// FLAGS: N Z
// Mode: Zero Page
// Syntax: LDA $44
// Hex: $A5
// Width: 2
// Timing: 2
fn execute_lda_z(system: &mut System) -> usize {
    let arg = system.next_byte() as u16;
    let value = system.memory_get(arg);
    system.chip.z = value == 0;
    system.chip.n = (value as i8) < 0;

    system.chip.a = value;
    2
}

// LDA
// FLAGS: N Z
// Mode: Absolute
// Syntax: LDA $4400
// Hex: $AD
// Width: 3
// Timing: 4
fn execute_lda_a(system: &mut System) -> usize {
    let low = system.next_byte() as u16;
    let high = system.next_byte() as u16;
    let value = system.memory_get((high << 8) + low);
    system.chip.z = value == 0;
    system.chip.n = (value as i8) < 0;

    system.chip.a = value;
    4
}

// LDA
// FLAGS: N Z
// Mode: Absolute,Y
// Syntax: LDA $4400,Y
// Hex: $B9
// Width: 3
// Timing: 4, +1 if page boundary crossed
fn execute_lda_ay(system: &mut System) -> usize {
    let low = system.next_byte() as u16;
    let high = system.next_byte() as u16;
    let addr = (high << 8) + low;
    let value = system.memory_get(addr + (system.chip.y as u16));
    system.chip.z = value == 0;
    system.chip.n = (value as i8) < 0;

    system.chip.a = value;
    4
}

// STA
// FLAGS: None
// Mode: Zero Page
// Syntax: STA $44
// Hex: $85
// Width: 2
// Timing: 3
fn execute_sta_z(system: &mut System) -> usize {
    let arg = system.next_byte();
    system.memory_set(arg as u16, system.chip.a);
    3
}

// STA
// FLAGS: None
// Mode: Zero Page,X
// Syntax: STA $44,X
// Hex: $95
// Width: 2
// Timing: 4
fn execute_sta_zx(system: &mut System) -> usize {
    let arg = system.next_byte();
    let index = (arg + system.chip.x) as u16;
    system.memory_set(index, system.chip.a);
    4
}

// STA
// FLAGS: None
// Mode: Absolute
// Syntax: STA $44,X
// Hex: $8D
// Width: 3
// Timing: 4
fn execute_sta_a(system: &mut System) -> usize {
    let low = system.next_byte() as u16;
    let high = system.next_byte() as u16;
    let index = (high << 8) + low;
    system.memory_set(index, system.chip.a);
    4
}

// STX
// FLAGS: None
// Mode: Absolute
// Syntax: STX $4444
// Hex: $8E
// Width: 3
// Timing: 4
fn execute_stx_a(system: &mut System) -> usize {
    let low = system.next_byte() as u16;
    let high = system.next_byte() as u16;
    let addr = (high << 8) + low;
    system.memory_set(addr, system.chip.x);
    4
}

// INX
// FLAGS: N Z
// Syntax: INX
// Hex: $E8
// Width: 1
// Timing: 2
fn execute_inx(system: &mut System) -> usize {
    system.chip.x = system.chip.x.wrapping_add(1);

    system.chip.z = system.chip.x == 0;
    system.chip.n = (system.chip.x as i8) < 0;
    2
}

// DEX
// FLAGS: N Z
// Syntax: DEX
// Hex: $CA
// Width: 1
// Timing: 2
fn execute_dex(system: &mut System) -> usize {
    system.chip.x = system.chip.x.wrapping_sub(1);

    system.chip.z = system.chip.x == 0;
    system.chip.n = (system.chip.x as i8) < 0;
    2
}

// Branching Instructions
//
// BNE
// FLAGS:
// Syntax: BNE Label
// Hex: $D0
// Width: 2
// Timing: 2, +1 if taken, +1 if across page boundary
fn execute_bne(system: &mut System) -> usize {
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

// BMI
// FLAGS:
// Syntax: BNE Label
// Hex: $30
// Width: 2
// Timing: 2, +1 if taken, +1 if across page boundary
fn execute_bmi(system: &mut System) -> usize {
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

// JMP
// FLAGS:
// Syntax: JMP Label
// Hex: $4C
// Width: 3
// Timing: 3
fn execute_jmp(system: &mut System) -> usize {
    let low = system.next_byte() as u16;
    let high = system.next_byte() as u16;
    system.chip.pc = (high << 8) + low;
    3
}

// Stack Instructions
//
// TXS
// FLAGS:
// Syntax: TXS
// Hex: $9A
// Width: 1
// Timing: 2
fn execute_txs(system: &mut System) -> usize {
    system.chip.sp = system.chip.x;
    2
}

// Register Instructions
//
// TXA
// FLAGS: N Z
// Syntax: TXA
// Hex: $8A
// Width: 1
// Timing: 2
fn execute_txa(system: &mut System) -> usize {
    system.chip.a = system.chip.x;
    system.chip.z = system.chip.a == 0;
    system.chip.n = (system.chip.a as i8) < 0;
    2
}

// TAY
// FLAGS: N Z
// Syntax: TAY
// Hex: $A8
// Width: 1
// Timing: 2
fn execute_tay(system: &mut System) -> usize {
    system.chip.y = system.chip.a;
    system.chip.z = system.chip.y == 0;
    system.chip.n = (system.chip.y as i8) < 0;
    2
}

// Subroutine Instructions
//
// JSR
// FLAGS:
// Syntax: JSR Label
// Hex: $20
// Width: 3
// Timing: 6
fn execute_jsr(system: &mut System) -> usize {
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

// RTS
// FLAGS:
// Syntax: RTS
// Hex: $60
// Width: 1
// Timing: 6
fn execute_rts(system: &mut System) -> usize {
    system.chip.sp += 1;
    let low = system.memory_get(system.chip.sp as u16) as u16;
    system.chip.sp += 1;
    let high = system.memory_get(system.chip.sp as u16) as u16;
    system.chip.pc = (high << 8) + low + 1;
    6
}

// Bitwise Operations

// EOR
// FLAGS: N Z
// Syntax: EOR #$44
// Mode: Immediate
// Hex: $49
// Width: 2
// Timing: 2
fn execute_eor_i(system: &mut System) -> usize {
    let arg = system.next_byte();
    system.chip.n = (arg as i8) < 0;
    system.chip.z = arg == 0;
    system.chip.a ^= arg;
    2
}

// LSR
// FLAGS: N Z C
// Syntax: LSR (A)
// Mode: Accumulator
// Hex: $4A
// Width: 1
// Timing: 2
fn execute_lsr_acc(system: &mut System) -> usize {
    let arg = system.next_byte();
    system.chip.c = system.chip.a & 0x01 == 0;
    system.chip.a >>= 1;
    system.chip.a ^= arg;
    2
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
fn execute_nop(_system: &mut System) -> usize {
    2
}

#[derive(Debug)]
pub enum InstructionName {
    Adc,
    Add,
    Asl,
    Bit,
    Bpl,
    Bmi,
    Bvc,
    Bvs,
    Bcc,
    Bcs,
    Bne,
    Beq,
    Brk,
    Cmp,
    Cpx,
    Cpy,
    Dec,
    Eor,
    Clc,
    Sec,
    Cli,
    Sei,
    Clv,
    Cld,
    Sed,
    Inc,
    Jmp,
    Jsr,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora,
    Tax,
    Txa,
    Dex,
    Inx,
    Tay,
    Tya,
    Dey,
    Iny,
    Rol,
    Ror,
    Rti,
    Rts,
    Sbc,
    Sta,
    Txs,
    Tsx,
    Pha,
    Pla,
    Php,
    Plp,
    Stx,
    Sty,
}

impl fmt::Display for InstructionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Adc => "ADC".to_owned(),
            Self::Add => "ADD".to_owned(),
            Self::Asl => "ASL".to_owned(),
            Self::Bit => "BIT".to_owned(),
            Self::Bpl => "BPL".to_owned(),
            Self::Bmi => "BMI".to_owned(),
            Self::Bvc => "BVC".to_owned(),
            Self::Bvs => "BVS".to_owned(),
            Self::Bcc => "BCC".to_owned(),
            Self::Bcs => "BCS".to_owned(),
            Self::Bne => "BNE".to_owned(),
            Self::Beq => "BEQ".to_owned(),
            Self::Brk => "BRK".to_owned(),
            Self::Cmp => "CMP".to_owned(),
            Self::Cpx => "CPX".to_owned(),
            Self::Cpy => "CPY".to_owned(),
            Self::Dec => "DEC".to_owned(),
            Self::Eor => "EOR".to_owned(),
            Self::Clc => "CLC".to_owned(),
            Self::Sec => "SEC".to_owned(),
            Self::Cli => "CLI".to_owned(),
            Self::Sei => "SEI".to_owned(),
            Self::Clv => "CLV".to_owned(),
            Self::Cld => "CLD".to_owned(),
            Self::Sed => "SED".to_owned(),
            Self::Inc => "INC".to_owned(),
            Self::Jmp => "JMP".to_owned(),
            Self::Jsr => "JSR".to_owned(),
            Self::Lda => "LDA".to_owned(),
            Self::Ldx => "LDX".to_owned(),
            Self::Ldy => "LDY".to_owned(),
            Self::Lsr => "LSR".to_owned(),
            Self::Nop => "NOP".to_owned(),
            Self::Ora => "ORA".to_owned(),
            Self::Tax => "TAX".to_owned(),
            Self::Txa => "TXA".to_owned(),
            Self::Dex => "DEX".to_owned(),
            Self::Inx => "INX".to_owned(),
            Self::Tay => "TAY".to_owned(),
            Self::Tya => "TYA".to_owned(),
            Self::Dey => "DEY".to_owned(),
            Self::Iny => "INY".to_owned(),
            Self::Rol => "ROL".to_owned(),
            Self::Ror => "ROR".to_owned(),
            Self::Rti => "RTI".to_owned(),
            Self::Rts => "RTS".to_owned(),
            Self::Sbc => "SBC".to_owned(),
            Self::Sta => "STA".to_owned(),
            Self::Txs => "TXS".to_owned(),
            Self::Tsx => "TSX".to_owned(),
            Self::Pha => "PHA".to_owned(),
            Self::Pla => "PLA".to_owned(),
            Self::Php => "PHP".to_owned(),
            Self::Plp => "PLP".to_owned(),
            Self::Stx => "STX".to_owned(),
            Self::Sty => "STY".to_owned(),
        };
        write!(f, "{}", name)
    }
}

// 1. Absolute a 4 (3) 4 (3) 3 3
// 3. Absolute Indexed with X a,x 4 (1,3) 4 (1,3) 3 3
// 4. Absolute Indexed with Y a,y 4 (1) 4 (1) 3 3
// 5. Absolute Indirect (a) 5 6 3 3
// 6. Accumulator A 2 2 1 1
// 7. Immediate # 2 2 2 2
// 8. Implied i 2 2 1 1
// 9. Program Counter Relative r 2 (1,2) 2 (1,2) 1 1
// 10. Stack s 3-7 3-7 1 1
// 11. Zero Page zp 3 (3) 3 (3) 2 2
// 12. Zero Page Indexed Indirect (zp,x) 6 6 2 2
// 13. Zero Page Indexed with X zp,x 4 (3) 4 (3) 2 2
// 14. Zero Page Indexed with Y zp,y 4 4 2 2
// 16. Zero Page Indirect Indexed with Y (zp),y
#[derive(Debug)]
pub enum AddressMode {
    Absolute,
    AbsoluteX,
    AbsoluteY,
    AbsoluteI,
    Accumulator,
    Immediate,
    Implied,
    Relative,
    ZeroPage,
    ZeroPageIX,
    ZeroPageY,
    ZeroPageX,
    ZeroPageIY,
}

impl AddressMode {}

pub struct InstructionValue {
    pub name: InstructionName,
    pub address_mode: AddressMode,
    execute: Option<fn(&mut System) -> usize>,
}

impl TryFrom<u8> for InstructionValue {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            // ADC
            0x69 => Self {
                name: InstructionName::Adc,
                address_mode: AddressMode::Immediate,
                execute: None,
            },
            0x65 => Self {
                name: InstructionName::Adc,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0x75 => Self {
                name: InstructionName::Adc,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0x6D => Self {
                name: InstructionName::Adc,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0x7D => Self {
                name: InstructionName::Adc,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            0x79 => Self {
                name: InstructionName::Adc,
                address_mode: AddressMode::AbsoluteY,
                execute: None,
            },
            0x61 => Self {
                name: InstructionName::Adc,
                address_mode: AddressMode::ZeroPageIX,
                execute: None,
            },
            0x71 => Self {
                name: InstructionName::Adc,
                address_mode: AddressMode::ZeroPageIY,
                execute: None,
            },
            // AND
            0x29 => Self {
                name: InstructionName::Add,
                address_mode: AddressMode::Immediate,
                execute: None,
            },
            0x25 => Self {
                name: InstructionName::Add,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0x35 => Self {
                name: InstructionName::Add,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0x2D => Self {
                name: InstructionName::Add,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0x3D => Self {
                name: InstructionName::Add,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            0x39 => Self {
                name: InstructionName::Add,
                address_mode: AddressMode::AbsoluteY,
                execute: None,
            },
            0x21 => Self {
                name: InstructionName::Add,
                address_mode: AddressMode::ZeroPageIX,
                execute: None,
            },
            0x31 => Self {
                name: InstructionName::Add,
                address_mode: AddressMode::ZeroPageIY,
                execute: None,
            },
            // ASL
            0x0A => Self {
                name: InstructionName::Asl,
                address_mode: AddressMode::Accumulator,
                execute: None,
            },
            0x06 => Self {
                name: InstructionName::Asl,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0x16 => Self {
                name: InstructionName::Asl,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0x0E => Self {
                name: InstructionName::Asl,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0x1E => Self {
                name: InstructionName::Asl,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            // BIT
            0x24 => Self {
                name: InstructionName::Bit,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0x2C => Self {
                name: InstructionName::Bit,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            // Branch
            0x10 => Self {
                name: InstructionName::Bpl,
                address_mode: AddressMode::Relative,
                execute: None,
            },
            0x30 => Self {
                name: InstructionName::Bmi,
                address_mode: AddressMode::Relative,
                execute: Some(execute_bmi),
            },
            0x50 => Self {
                name: InstructionName::Bvc,
                address_mode: AddressMode::Relative,
                execute: None,
            },
            0x70 => Self {
                name: InstructionName::Bvs,
                address_mode: AddressMode::Relative,
                execute: None,
            },
            0x90 => Self {
                name: InstructionName::Bcc,
                address_mode: AddressMode::Relative,
                execute: None,
            },
            0xB0 => Self {
                name: InstructionName::Bcs,
                address_mode: AddressMode::Relative,
                execute: None,
            },
            0xD0 => Self {
                name: InstructionName::Bne,
                address_mode: AddressMode::Relative,
                execute: Some(execute_bne),
            },
            0xF0 => Self {
                name: InstructionName::Beq,
                address_mode: AddressMode::Relative,
                execute: None,
            },
            // BRK
            0x00 => Self {
                name: InstructionName::Brk,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            // CMP
            0xc9 => Self {
                name: InstructionName::Cmp,
                address_mode: AddressMode::Immediate,
                execute: None,
            },
            0xc5 => Self {
                name: InstructionName::Cmp,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0xD5 => Self {
                name: InstructionName::Cmp,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0xCD => Self {
                name: InstructionName::Cmp,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0xDD => Self {
                name: InstructionName::Cmp,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            0xD9 => Self {
                name: InstructionName::Cmp,
                address_mode: AddressMode::AbsoluteY,
                execute: None,
            },
            0xC1 => Self {
                name: InstructionName::Cmp,
                address_mode: AddressMode::ZeroPageIX,
                execute: None,
            },
            0xD1 => Self {
                name: InstructionName::Cmp,
                address_mode: AddressMode::ZeroPageIY,
                execute: None,
            },
            // CPX
            0xE0 => Self {
                name: InstructionName::Cpx,
                address_mode: AddressMode::Immediate,
                execute: None,
            },
            0xE4 => Self {
                name: InstructionName::Cpx,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0xEC => Self {
                name: InstructionName::Cpx,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            // CPY
            0xC0 => Self {
                name: InstructionName::Cpy,
                address_mode: AddressMode::Immediate,
                execute: None,
            },
            0xC4 => Self {
                name: InstructionName::Cpy,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0xCC => Self {
                name: InstructionName::Cpy,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            // DEC
            0xC6 => Self {
                name: InstructionName::Dec,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0xD6 => Self {
                name: InstructionName::Dec,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0xCE => Self {
                name: InstructionName::Dec,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0xDE => Self {
                name: InstructionName::Dec,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            // EOR
            0x49 => Self {
                name: InstructionName::Eor,
                address_mode: AddressMode::Immediate,
                execute: Some(execute_eor_i),
            },
            0x45 => Self {
                name: InstructionName::Eor,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0x55 => Self {
                name: InstructionName::Eor,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0x4D => Self {
                name: InstructionName::Eor,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0x5D => Self {
                name: InstructionName::Eor,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            0x59 => Self {
                name: InstructionName::Eor,
                address_mode: AddressMode::AbsoluteY,
                execute: None,
            },
            0x41 => Self {
                name: InstructionName::Eor,
                address_mode: AddressMode::ZeroPageIX,
                execute: None,
            },
            0x51 => Self {
                name: InstructionName::Eor,
                address_mode: AddressMode::ZeroPageIY,
                execute: None,
            },
            // Flag Instruction
            0x18 => Self {
                name: InstructionName::Clc,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0x38 => Self {
                name: InstructionName::Sec,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0x58 => Self {
                name: InstructionName::Cli,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0x78 => Self {
                name: InstructionName::Sei,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0xB8 => Self {
                name: InstructionName::Clv,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0xD8 => Self {
                name: InstructionName::Cld,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0xF8 => Self {
                name: InstructionName::Sed,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            // INC
            0xE6 => Self {
                name: InstructionName::Inc,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0xF6 => Self {
                name: InstructionName::Inc,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0xEE => Self {
                name: InstructionName::Inc,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0xFE => Self {
                name: InstructionName::Inc,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            // JMP
            0x4C => Self {
                name: InstructionName::Jmp,
                address_mode: AddressMode::Absolute,
                execute: Some(execute_jmp),
            },
            0x6C => Self {
                name: InstructionName::Jmp,
                address_mode: AddressMode::AbsoluteI,
                execute: None,
            },
            // JSR
            0x20 => Self {
                name: InstructionName::Jsr,
                address_mode: AddressMode::Absolute,
                execute: Some(execute_jsr),
            },
            // LDA
            0xA9 => Self {
                name: InstructionName::Lda,
                address_mode: AddressMode::Immediate,
                execute: Some(execute_lda_i),
            },
            0xA5 => Self {
                name: InstructionName::Lda,
                address_mode: AddressMode::ZeroPage,
                execute: Some(execute_lda_z),
            },
            0xB5 => Self {
                name: InstructionName::Lda,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0xAD => Self {
                name: InstructionName::Lda,
                address_mode: AddressMode::Absolute,
                execute: Some(execute_lda_a),
            },
            0xBD => Self {
                name: InstructionName::Lda,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            0xB9 => Self {
                name: InstructionName::Lda,
                address_mode: AddressMode::AbsoluteY,
                execute: Some(execute_lda_ay),
            },
            0xA1 => Self {
                name: InstructionName::Lda,
                address_mode: AddressMode::ZeroPageIX,
                execute: None,
            },
            0xB1 => Self {
                name: InstructionName::Lda,
                address_mode: AddressMode::ZeroPageIY,
                execute: None,
            },
            // LDX
            0xA2 => Self {
                name: InstructionName::Ldx,
                address_mode: AddressMode::Immediate,
                execute: Some(execute_ldx_i),
            },
            0xA6 => Self {
                name: InstructionName::Ldx,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0xB6 => Self {
                name: InstructionName::Ldx,
                address_mode: AddressMode::ZeroPageY,
                execute: None,
            },
            0xAE => Self {
                name: InstructionName::Ldx,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0xBE => Self {
                name: InstructionName::Ldx,
                address_mode: AddressMode::AbsoluteY,
                execute: None,
            },
            // LDY
            0xA0 => Self {
                name: InstructionName::Ldy,
                address_mode: AddressMode::Immediate,
                execute: None,
            },
            0xA4 => Self {
                name: InstructionName::Ldy,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0xB4 => Self {
                name: InstructionName::Ldy,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0xAC => Self {
                name: InstructionName::Ldy,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0xBC => Self {
                name: InstructionName::Ldy,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            // LSR
            0x4A => Self {
                name: InstructionName::Lsr,
                address_mode: AddressMode::Accumulator,
                execute: Some(execute_lsr_acc),
            },
            0x46 => Self {
                name: InstructionName::Lsr,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0x56 => Self {
                name: InstructionName::Lsr,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0x4E => Self {
                name: InstructionName::Lsr,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0x5E => Self {
                name: InstructionName::Lsr,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            // Nop
            0xEA => Self {
                name: InstructionName::Nop,
                address_mode: AddressMode::Implied,
                execute: Some(execute_nop),
            },
            // ORA
            0x09 => Self {
                name: InstructionName::Ora,
                address_mode: AddressMode::Immediate,
                execute: None,
            },
            0x05 => Self {
                name: InstructionName::Ora,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0x15 => Self {
                name: InstructionName::Ora,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0x0D => Self {
                name: InstructionName::Ora,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0x1D => Self {
                name: InstructionName::Ora,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            0x19 => Self {
                name: InstructionName::Ora,
                address_mode: AddressMode::AbsoluteY,
                execute: None,
            },
            0x01 => Self {
                name: InstructionName::Ora,
                address_mode: AddressMode::ZeroPageIX,
                execute: None,
            },
            0x11 => Self {
                name: InstructionName::Ora,
                address_mode: AddressMode::ZeroPageIY,
                execute: None,
            },
            // Register Instructions
            0xAA => Self {
                name: InstructionName::Tax,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0x8A => Self {
                name: InstructionName::Txa,
                address_mode: AddressMode::Implied,
                execute: Some(execute_txa),
            },
            0xCA => Self {
                name: InstructionName::Dex,
                address_mode: AddressMode::Implied,
                execute: Some(execute_dex),
            },
            0xE8 => Self {
                name: InstructionName::Inx,
                address_mode: AddressMode::Implied,
                execute: Some(execute_inx),
            },
            0xA8 => Self {
                name: InstructionName::Tay,
                address_mode: AddressMode::Implied,
                execute: Some(execute_tay),
            },
            0x98 => Self {
                name: InstructionName::Tya,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0x88 => Self {
                name: InstructionName::Dey,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0xC8 => Self {
                name: InstructionName::Iny,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            // ROL
            0x2A => Self {
                name: InstructionName::Rol,
                address_mode: AddressMode::Accumulator,
                execute: None,
            },
            0x26 => Self {
                name: InstructionName::Rol,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0x36 => Self {
                name: InstructionName::Rol,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0x2E => Self {
                name: InstructionName::Rol,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0x3E => Self {
                name: InstructionName::Rol,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            // ROR
            0x6A => Self {
                name: InstructionName::Ror,
                address_mode: AddressMode::Accumulator,
                execute: None,
            },
            0x66 => Self {
                name: InstructionName::Ror,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0x76 => Self {
                name: InstructionName::Ror,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0x6E => Self {
                name: InstructionName::Ror,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0x7E => Self {
                name: InstructionName::Ror,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            // RTI
            0x40 => Self {
                name: InstructionName::Rti,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            // RTS
            0x60 => Self {
                name: InstructionName::Rts,
                address_mode: AddressMode::Implied,
                execute: Some(execute_rts),
            },
            // SBC
            0xE9 => Self {
                name: InstructionName::Sbc,
                address_mode: AddressMode::Immediate,
                execute: None,
            },
            0xE5 => Self {
                name: InstructionName::Sbc,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0xF5 => Self {
                name: InstructionName::Sbc,
                address_mode: AddressMode::ZeroPageX,
                execute: None,
            },
            0xED => Self {
                name: InstructionName::Sbc,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            0xFD => Self {
                name: InstructionName::Sbc,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            0xF9 => Self {
                name: InstructionName::Sbc,
                address_mode: AddressMode::AbsoluteY,
                execute: None,
            },
            0xE1 => Self {
                name: InstructionName::Sbc,
                address_mode: AddressMode::ZeroPageIX,
                execute: None,
            },
            0xF1 => Self {
                name: InstructionName::Sbc,
                address_mode: AddressMode::ZeroPageIY,
                execute: None,
            },
            // STA
            0x85 => Self {
                name: InstructionName::Sta,
                address_mode: AddressMode::ZeroPage,
                execute: Some(execute_sta_z),
            },
            0x95 => Self {
                name: InstructionName::Sta,
                address_mode: AddressMode::ZeroPageX,
                execute: Some(execute_sta_zx),
            },
            0x8D => Self {
                name: InstructionName::Sta,
                address_mode: AddressMode::Absolute,
                execute: Some(execute_sta_a),
            },
            0x9D => Self {
                name: InstructionName::Sta,
                address_mode: AddressMode::AbsoluteX,
                execute: None,
            },
            0x99 => Self {
                name: InstructionName::Sta,
                address_mode: AddressMode::AbsoluteY,
                execute: None,
            },
            0x81 => Self {
                name: InstructionName::Sta,
                address_mode: AddressMode::ZeroPageIX,
                execute: None,
            },
            0x91 => Self {
                name: InstructionName::Sta,
                address_mode: AddressMode::ZeroPageIY,
                execute: None,
            },
            // Stack Instructions
            0x9A => Self {
                name: InstructionName::Txs,
                address_mode: AddressMode::Implied,
                execute: Some(execute_txs),
            },
            0xBA => Self {
                name: InstructionName::Tsx,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0x48 => Self {
                name: InstructionName::Pha,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0x68 => Self {
                name: InstructionName::Pla,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0x08 => Self {
                name: InstructionName::Php,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            0x28 => Self {
                name: InstructionName::Plp,
                address_mode: AddressMode::Implied,
                execute: None,
            },
            // STX
            0x86 => Self {
                name: InstructionName::Stx,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0x96 => Self {
                name: InstructionName::Stx,
                address_mode: AddressMode::ZeroPageY,
                execute: None,
            },
            0x8E => Self {
                name: InstructionName::Stx,
                address_mode: AddressMode::Absolute,
                execute: Some(execute_stx_a),
            },
            // STY
            0x84 => Self {
                name: InstructionName::Sty,
                address_mode: AddressMode::ZeroPage,
                execute: None,
            },
            0x94 => Self {
                name: InstructionName::Sty,
                address_mode: AddressMode::ZeroPageY,
                execute: None,
            },
            0x8C => Self {
                name: InstructionName::Sty,
                address_mode: AddressMode::Absolute,
                execute: None,
            },
            _ => return Err(format!("Unknown instruction: {:02X}", value)),
        })
    }
}

impl InstructionValue {
    pub fn execute(&self, system: &mut System) -> crate::Result<usize> {
        let execute = self.execute.ok_or_else(|| {
            format!(
                "Execute not implemented for instruction: {:?} addressing mode: {:?}",
                self.name, self.address_mode
            )
        })?;
        Ok(execute(system))
    }

    pub fn format_arguments<'a, T>(
        &self,
        iter: &mut T,
        symbol_map: &HashMap<u16, String>,
        pc: u16,
    ) -> String
    where
        T: Iterator<Item = (usize, &'a u8)>,
    {
        match self.address_mode {
            AddressMode::Absolute => {
                let low = *iter.next().unwrap().1 as u16;
                let high = *iter.next().unwrap().1 as u16;
                let addr = (high << 8) + low;
                symbol_map
                    .get(&(addr & 0x1FFF))
                    .map(|sym| sym.to_owned())
                    .unwrap_or_else(|| format!("${addr:04X}"))
            }
            AddressMode::AbsoluteX => {
                let low = *iter.next().unwrap().1 as u16;
                let high = *iter.next().unwrap().1 as u16;
                let addr = (high << 8) + low;
                let addr = symbol_map
                    .get(&(addr & 0x1FFF))
                    .map(|sym| sym.to_owned())
                    .unwrap_or_else(|| format!("${addr:04X}"));
                format!("{addr}, X")
            }
            AddressMode::AbsoluteY => {
                let low = *iter.next().unwrap().1 as u16;
                let high = *iter.next().unwrap().1 as u16;
                let addr = (high << 8) + low;
                let addr = symbol_map
                    .get(&(addr & 0x1FFF))
                    .map(|sym| sym.to_owned())
                    .unwrap_or_else(|| format!("${addr:04X}"));
                format!("{addr}, Y")
            }
            AddressMode::AbsoluteI => {
                let low = *iter.next().unwrap().1 as u16;
                let high = *iter.next().unwrap().1 as u16;
                let addr = (high << 8) + low;
                let addr = symbol_map
                    .get(&(addr & 0x1FFF))
                    .map(|sym| sym.to_owned())
                    .unwrap_or_else(|| format!("${addr:04X}"));
                format!("({addr})")
            }
            AddressMode::Accumulator | AddressMode::Implied => "".to_owned(),
            AddressMode::Immediate => {
                let value = *iter.next().unwrap().1 as u16;
                format!("#${value:02X}")
            }
            AddressMode::Relative => {
                let value = *iter.next().unwrap().1 as i8;
                let addr = (pc + 2).wrapping_add(value as u16);
                symbol_map
                    .get(&(addr & 0x1FFF))
                    .map(|sym| sym.to_owned())
                    .unwrap_or_else(|| format!("${addr:02X}"))
            }
            AddressMode::ZeroPage => {
                let addr = *iter.next().unwrap().1 as u16;
                symbol_map
                    .get(&(addr & 0x1FFF))
                    .map(|sym| sym.to_owned())
                    .unwrap_or_else(|| format!("${addr:02X}"))
            }
            AddressMode::ZeroPageIX => {
                let addr = *iter.next().unwrap().1 as u16;
                let addr = symbol_map
                    .get(&(addr & 0x1FFF))
                    .map(|sym| sym.to_owned())
                    .unwrap_or_else(|| format!("${addr:02X}"));
                format!("(${addr}, X)")
            }
            AddressMode::ZeroPageY => {
                let addr = *iter.next().unwrap().1 as u16;
                let addr = symbol_map
                    .get(&(addr & 0x1FFF))
                    .map(|sym| sym.to_owned())
                    .unwrap_or_else(|| format!("${addr:02X}"));
                format!("${addr}, Y")
            }
            AddressMode::ZeroPageX => {
                let addr = *iter.next().unwrap().1 as u16;
                let addr = symbol_map
                    .get(&(addr & 0x1FFF))
                    .map(|sym| sym.to_owned())
                    .unwrap_or_else(|| format!("${addr:02X}"));
                format!("${addr}, X")
            }
            AddressMode::ZeroPageIY => {
                let addr = *iter.next().unwrap().1 as u16;
                let addr = symbol_map
                    .get(&(addr & 0x1FFF))
                    .map(|sym| sym.to_owned())
                    .unwrap_or_else(|| format!("${addr:02X}"));
                format!("(${addr}), Y")
            }
        }
    }
}

impl fmt::Display for InstructionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
