use std::collections::HashMap;
use std::fmt;

use super::System;

#[derive(Debug)]
pub enum Instruction {
    Adc(AddressMode),
    And(AddressMode),
    Asl(AddressMode),
    Bit(AddressMode),
    Bpl(AddressMode),
    Bmi(AddressMode),
    Bvc(AddressMode),
    Bvs(AddressMode),
    Bcc(AddressMode),
    Bcs(AddressMode),
    Bne(AddressMode),
    Beq(AddressMode),
    Brk(AddressMode),
    Cmp(AddressMode),
    Cpx(AddressMode),
    Cpy(AddressMode),
    Dec(AddressMode),
    Eor(AddressMode),
    Clc(AddressMode),
    Sec(AddressMode),
    Cli(AddressMode),
    Sei(AddressMode),
    Clv(AddressMode),
    Cld(AddressMode),
    Sed(AddressMode),
    Inc(AddressMode),
    Jmp(AddressMode),
    Jsr(AddressMode),
    Lda(AddressMode),
    Ldx(AddressMode),
    Ldy(AddressMode),
    Lsr(AddressMode),
    Nop(AddressMode),
    Ora(AddressMode),
    Tax(AddressMode),
    Txa(AddressMode),
    Dex(AddressMode),
    Inx(AddressMode),
    Tay(AddressMode),
    Tya(AddressMode),
    Dey(AddressMode),
    Iny(AddressMode),
    Rol(AddressMode),
    Ror(AddressMode),
    Rti(AddressMode),
    Rts(AddressMode),
    Sbc(AddressMode),
    Sta(AddressMode),
    Txs(AddressMode),
    Tsx(AddressMode),
    Pha(AddressMode),
    Pla(AddressMode),
    Php(AddressMode),
    Plp(AddressMode),
    Stx(AddressMode),
    Sty(AddressMode),
    // Illegal opcodes
    Dop(AddressMode),
}

impl Instruction {
    pub fn execute(&self, system: &mut System) -> crate::Result<usize> {
        let mut clocks = 0;

        match self {
            // TODO: Decimal mode
            Self::Adc(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                clocks += 1;
                let value = match address_value {
                    AddressValue::Value(val) => val,
                    AddressValue::Address {
                        addr,
                        page_boundary_crossed,
                        ..
                    } => {
                        clocks += page_boundary_crossed as usize;
                        system.memory_get(addr)
                    }
                    _ => unreachable!(),
                };
                let a = system.chip.a as u16;
                let v = value as u16;
                let c = system.chip.c as u16;
                let result = a + v + c;
                system.chip.c = result > 0xFF;
                // Overflow is only set if the result is a different sign from both of the operands
                // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
                system.chip.v = (a ^ result) & (v ^ result) & 0x80 != 0;
                system.chip.n = result & 0x80 != 0;

                let result = result as u8;
                system.chip.z = result == 0;
                system.chip.a = result;
            }
            // TODO: Decimal mode
            Self::Sbc(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                clocks += 1;
                let value = match address_value {
                    AddressValue::Value(val) => val,
                    AddressValue::Address {
                        addr,
                        page_boundary_crossed,
                        ..
                    } => {
                        clocks += page_boundary_crossed as usize;
                        system.memory_get(addr)
                    }
                    _ => unreachable!(),
                };
                let a = system.chip.a;
                let v = value;
                let c = system.chip.c as u8;
                let result = a.wrapping_add(!v).wrapping_add(c);
                system.chip.c = result & 0x80 != 0;
                system.chip.v = (a ^ result) & ((!v) ^ result) & 0x80 != 0;
                system.chip.n = result & 0x80 != 0;

                let result = result as u8;
                system.chip.z = result == 0;
                system.chip.a = result;
            }
            Self::And(mode) | Self::Ora(mode) | Self::Eor(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                clocks += 1;
                let value = match address_value {
                    AddressValue::Value(val) => val,
                    AddressValue::Address {
                        addr,
                        page_boundary_crossed,
                        ..
                    } => {
                        clocks += page_boundary_crossed as usize;
                        system.memory_get(addr)
                    }
                    _ => unreachable!(),
                };
                system.chip.a = match self {
                    Self::And(_) => system.chip.a & value,
                    Self::Ora(_) => system.chip.a | value,
                    Self::Eor(_) => system.chip.a ^ value,
                    _ => unreachable!(),
                };
                system.chip.z = system.chip.a == 0;
                system.chip.n = system.chip.a & 0x80 != 0;
            }
            Self::Asl(mode) | Self::Lsr(mode) | Self::Rol(mode) | Self::Ror(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                let carry = system.chip.c as u8;
                let mut c = false;
                let mut z = false;
                let mut n = false;

                let mut calc = |val: u8| {
                    let (value, carry) = match self {
                        Self::Asl(_) => (((val as u16) << 1) as u8, val & 0x80 != 0),
                        Self::Lsr(_) => (val >> 1, val & 0x01 != 0),
                        Self::Rol(_) => {
                            let shifted = ((val as u16) << 1) as u8;
                            let shifted = shifted | carry;
                            (shifted, val & 0x80 != 0)
                        }
                        Self::Ror(_) => {
                            let shifted = (val >> 1) as u8;
                            let shifted = shifted | (carry << 7);
                            (shifted, val & 0x01 != 0)
                        }
                        _ => unreachable!(),
                    };
                    c = carry;
                    z = value == 0;
                    n = value & 0x80 != 0;
                    value
                };

                match address_value {
                    AddressValue::None => {
                        clocks += 2;
                        system.chip.a = calc(system.chip.a);
                    }
                    AddressValue::Address {
                        addr, is_offset, ..
                    } => {
                        clocks += 3 + is_offset as usize;
                        let val = {
                            let val = system.memory_get(addr);
                            calc(val)
                        };
                        system.memory_set(addr, val);
                    }
                    _ => unreachable!(),
                }
                system.chip.c = c;
                system.chip.z = z;
                system.chip.n = n;
            }
            Self::Bit(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                clocks += 1;
                let value = match address_value {
                    AddressValue::Address { addr, .. } => system.memory_get(addr),
                    _ => unreachable!(),
                };
                let test_value = system.chip.a & value;
                system.chip.z = test_value == 0;
                system.chip.n = test_value & 0x80 != 0;
                system.chip.v = test_value & 0x40 != 0;
            }
            Self::Bpl(mode)
            | Self::Bmi(mode)
            | Self::Bvc(mode)
            | Self::Bvs(mode)
            | Self::Bcc(mode)
            | Self::Bcs(mode)
            | Self::Bne(mode)
            | Self::Beq(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                clocks += 2;
                let should_branch = match self {
                    Self::Bpl(_) => !system.chip.n,
                    Self::Bmi(_) => system.chip.n,
                    Self::Bvc(_) => !system.chip.v,
                    Self::Bvs(_) => system.chip.v,
                    Self::Bcc(_) => !system.chip.c,
                    Self::Bcs(_) => system.chip.c,
                    Self::Bne(_) => !system.chip.z,
                    Self::Beq(_) => system.chip.z,
                    _ => unreachable!(),
                };
                let addr = match address_value {
                    AddressValue::Address { addr, .. } => addr,
                    _ => unreachable!(),
                };

                if should_branch {
                    clocks += 1;
                    clocks += (system.chip.pc & 0xFF00 != addr & 0xFF00) as usize;
                    system.chip.pc = addr;
                }
            }
            Self::Brk(_) | Self::Rti(_) => {
                // Break is special. My basic understanding is that it is used to cause
                // program-controlled irq. It pushes the status register to the the stack and the
                // PC + 2. It can be used for some rare, but interesting tricks.
                // See: http://archive.6502.org/books/mcs6500_family_programming_manual.pdf page
                // 144 for details and examples.
                unimplemented!("BRK and RTI not implemented -- save for a fun stream topic")
            }
            Self::Cmp(mode) | Self::Cpx(mode) | Self::Cpy(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                clocks += 1;
                let base = match self {
                    Self::Cmp(_) => system.chip.a,
                    Self::Cpx(_) => system.chip.x,
                    Self::Cpy(_) => system.chip.y,
                    _ => unreachable!(),
                };
                let value = match address_value {
                    AddressValue::Value(val) => val,
                    AddressValue::Address {
                        addr,
                        page_boundary_crossed,
                        ..
                    } => {
                        clocks += page_boundary_crossed as usize;
                        system.memory_get(addr)
                    }
                    _ => unreachable!(),
                };

                let result = base.wrapping_sub(value);
                system.chip.z = result == 0;
                system.chip.n = result & 0x80 != 0;
                system.chip.c = base > value;
            }
            Self::Dec(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                clocks += 3;
                let addr = match address_value {
                    AddressValue::Address {
                        addr, is_offset, ..
                    } => {
                        clocks += is_offset as usize;
                        addr
                    }
                    _ => unreachable!(),
                };
                let value = system.memory_get(addr);
                let result = value.wrapping_sub(1);
                system.chip.z = result == 0;
                system.chip.n = result & 0x80 != 0;
                system.memory_set(addr, result);
            }
            Self::Clc(_) => {
                clocks += 2;
                system.chip.c = false;
            }
            Self::Sec(_) => {
                clocks += 2;
                system.chip.c = true;
            }
            Self::Cli(_) => {
                clocks += 2;
                system.chip.i = false;
            }
            Self::Sei(_) => {
                clocks += 2;
                system.chip.i = true;
            }
            Self::Clv(_) => {
                clocks += 2;
                system.chip.v = false;
            }
            Self::Cld(_) => {
                clocks += 2;
                system.chip.d = false;
            }
            Self::Sed(_) => {
                clocks += 2;
                system.chip.d = true;
            }
            Self::Inc(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                clocks += 3;
                let addr = match address_value {
                    AddressValue::Address {
                        addr, is_offset, ..
                    } => {
                        clocks += is_offset as usize;
                        addr
                    }
                    _ => unreachable!(),
                };
                let value = system.memory_get(addr);
                let result = value.wrapping_add(1);
                system.chip.z = result == 0;
                system.chip.n = result & 0x80 != 0;
                system.memory_set(addr, result);
            }
            Self::Jmp(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                system.chip.pc = match address_value {
                    AddressValue::Address { addr, .. } => addr,
                    _ => unreachable!(),
                };
            }
            Self::Jsr(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                clocks += 3;
                let addr = match address_value {
                    AddressValue::Address { addr, .. } => addr,
                    _ => unreachable!(),
                };

                let ret_low = system.chip.pc as u8;
                let ret_high = (system.chip.pc >> 8) as u8;
                system.memory_set(system.chip.sp as u16, ret_high);
                system.chip.sp -= 1;
                system.memory_set(system.chip.sp as u16, ret_low - 1);
                system.chip.sp -= 1;
                system.chip.pc = addr;
            }
            Self::Lda(mode) | Self::Ldx(mode) | Self::Ldy(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                clocks += 1;
                let value = match address_value {
                    AddressValue::Value(val) => val,
                    AddressValue::Address {
                        addr,
                        page_boundary_crossed,
                        ..
                    } => {
                        clocks += page_boundary_crossed as usize;
                        system.memory_get(addr)
                    }
                    _ => unreachable!(),
                };
                let register = match self {
                    Self::Lda(_) => &mut system.chip.a,
                    Self::Ldx(_) => &mut system.chip.x,
                    Self::Ldy(_) => &mut system.chip.y,
                    _ => unreachable!(),
                };
                system.chip.z = value == 0;
                system.chip.n = value & 0x80 != 0;
                *register = value;
            }
            Self::Nop(_) => clocks += 2,
            Self::Tax(_) | Self::Txa(_) | Self::Tay(_) | Self::Tya(_) => {
                clocks += 2;
                let (source, dest) = match self {
                    Self::Tax(_) => (system.chip.a, &mut system.chip.x),
                    Self::Txa(_) => (system.chip.x, &mut system.chip.a),
                    Self::Tay(_) => (system.chip.a, &mut system.chip.y),
                    Self::Tya(_) => (system.chip.y, &mut system.chip.a),
                    _ => unreachable!(),
                };
                *dest = source;
                system.chip.z = source == 0;
                system.chip.n = source & 0x80 != 0;
            }
            Self::Dex(_) | Self::Dey(_) => {
                clocks += 2;
                let register = match self {
                    Self::Dex(_) => &mut system.chip.x,
                    Self::Dey(_) => &mut system.chip.y,
                    _ => unreachable!(),
                };
                *register = register.wrapping_sub(1);
                system.chip.z = *register == 0;
                system.chip.n = *register & 0x80 != 0;
            }
            Self::Inx(_) | Self::Iny(_) => {
                clocks += 2;
                let register = match self {
                    Self::Inx(_) => &mut system.chip.x,
                    Self::Iny(_) => &mut system.chip.y,
                    _ => unreachable!(),
                };
                *register = register.wrapping_add(1);
                system.chip.z = *register == 0;
                system.chip.n = *register & 0x80 != 0;
            }
            Self::Rts(_) => {
                clocks += 6;
                system.chip.sp += 1;
                let low = system.memory_get(system.chip.sp as u16) as u16;
                system.chip.sp += 1;
                let high = system.memory_get(system.chip.sp as u16) as u16;
                system.chip.pc = (high << 8) + low + 1;
            }
            Self::Txs(_) | Self::Tsx(_) => {
                clocks += 2;
                let (source, dest) = match self {
                    Self::Tsx(_) => (system.chip.sp, &mut system.chip.x),
                    Self::Txs(_) => (system.chip.x, &mut system.chip.sp),
                    _ => unreachable!(),
                };
                *dest = source;
            }
            Self::Pha(_) | Self::Php(_) => {
                clocks += 3;
                let value = match self {
                    Self::Pha(_) => system.chip.a,
                    Self::Php(_) => system.status(),
                    _ => unreachable!(),
                };
                system.memory_set(system.chip.sp as u16, value);
                system.chip.sp -= 1;
            }
            Self::Pla(_) => {
                clocks += 4;
                system.chip.sp += 1;
                system.chip.a = system.memory_get(system.chip.sp as u16);
            }
            Self::Plp(_) => {
                clocks += 4;
                system.chip.sp += 1;
                let register = system.memory_get(system.chip.sp as u16);
                system.status_set(register);
            }
            Self::Sta(mode) | Self::Stx(mode) | Self::Sty(mode) => {
                let address_value = mode.execute(system, &mut clocks);
                clocks += 1;
                let addr = match address_value {
                    AddressValue::Address {
                        addr, is_offset, ..
                    } => {
                        clocks += is_offset as usize;
                        addr
                    }
                    _ => unreachable!(),
                };
                let value = match self {
                    Self::Sta(_) => system.chip.a,
                    Self::Stx(_) => system.chip.x,
                    Self::Sty(_) => system.chip.y,
                    _ => unreachable!(),
                };

                system.memory_set(addr, value);
            }
            // Illegal opcodes
            Self::Dop(mode) => {
                mode.execute(system, &mut clocks);
                clocks += 1
            }
        }
        Ok(clocks)
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
        let mode = match self {
            Self::Adc(mode)
            | Self::And(mode)
            | Self::Asl(mode)
            | Self::Bit(mode)
            | Self::Bpl(mode)
            | Self::Bmi(mode)
            | Self::Bvc(mode)
            | Self::Bvs(mode)
            | Self::Bcc(mode)
            | Self::Bcs(mode)
            | Self::Bne(mode)
            | Self::Beq(mode)
            | Self::Brk(mode)
            | Self::Cmp(mode)
            | Self::Cpx(mode)
            | Self::Cpy(mode)
            | Self::Dec(mode)
            | Self::Eor(mode)
            | Self::Clc(mode)
            | Self::Sec(mode)
            | Self::Cli(mode)
            | Self::Sei(mode)
            | Self::Clv(mode)
            | Self::Cld(mode)
            | Self::Sed(mode)
            | Self::Inc(mode)
            | Self::Jmp(mode)
            | Self::Jsr(mode)
            | Self::Lda(mode)
            | Self::Ldx(mode)
            | Self::Ldy(mode)
            | Self::Lsr(mode)
            | Self::Nop(mode)
            | Self::Ora(mode)
            | Self::Tax(mode)
            | Self::Txa(mode)
            | Self::Dex(mode)
            | Self::Inx(mode)
            | Self::Tay(mode)
            | Self::Tya(mode)
            | Self::Dey(mode)
            | Self::Iny(mode)
            | Self::Rol(mode)
            | Self::Ror(mode)
            | Self::Rti(mode)
            | Self::Rts(mode)
            | Self::Sbc(mode)
            | Self::Sta(mode)
            | Self::Txs(mode)
            | Self::Tsx(mode)
            | Self::Pha(mode)
            | Self::Pla(mode)
            | Self::Php(mode)
            | Self::Plp(mode)
            | Self::Stx(mode)
            | Self::Sty(mode)
            // Illegal opcodes
            | Self::Dop(mode)=> mode,
        };

        match mode {
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

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Adc(_) => "ADC",
            Self::And(_) => "ADD",
            Self::Asl(_) => "ASL",
            Self::Bit(_) => "BIT",
            Self::Bpl(_) => "BPL",
            Self::Bmi(_) => "BMI",
            Self::Bvc(_) => "BVC",
            Self::Bvs(_) => "BVS",
            Self::Bcc(_) => "BCC",
            Self::Bcs(_) => "BCS",
            Self::Bne(_) => "BNE",
            Self::Beq(_) => "BEQ",
            Self::Brk(_) => "BRK",
            Self::Cmp(_) => "CMP",
            Self::Cpx(_) => "CPX",
            Self::Cpy(_) => "CPY",
            Self::Dec(_) => "DEC",
            Self::Eor(_) => "EOR",
            Self::Clc(_) => "CLC",
            Self::Sec(_) => "SEC",
            Self::Cli(_) => "CLI",
            Self::Sei(_) => "SEI",
            Self::Clv(_) => "CLV",
            Self::Cld(_) => "CLD",
            Self::Sed(_) => "SED",
            Self::Inc(_) => "INC",
            Self::Jmp(_) => "JMP",
            Self::Jsr(_) => "JSR",
            Self::Lda(_) => "LDA",
            Self::Ldx(_) => "LDX",
            Self::Ldy(_) => "LDY",
            Self::Lsr(_) => "LSR",
            Self::Nop(_) => "NOP",
            Self::Ora(_) => "ORA",
            Self::Tax(_) => "TAX",
            Self::Txa(_) => "TXA",
            Self::Dex(_) => "DEX",
            Self::Inx(_) => "INX",
            Self::Tay(_) => "TAY",
            Self::Tya(_) => "TYA",
            Self::Dey(_) => "DEY",
            Self::Iny(_) => "INY",
            Self::Rol(_) => "ROL",
            Self::Ror(_) => "ROR",
            Self::Rti(_) => "RTI",
            Self::Rts(_) => "RTS",
            Self::Sbc(_) => "SBC",
            Self::Sta(_) => "STA",
            Self::Txs(_) => "TXS",
            Self::Tsx(_) => "TSX",
            Self::Pha(_) => "PHA",
            Self::Pla(_) => "PLA",
            Self::Php(_) => "PHP",
            Self::Plp(_) => "PLP",
            Self::Stx(_) => "STX",
            Self::Sty(_) => "STY",
            // Illegal Opcodes
            Self::Dop(_) => "DOP",
        };
        write!(f, "{}", name.to_owned())
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

#[derive(Debug, PartialEq)]
pub enum AddressValue {
    Address {
        addr: u16,
        page_boundary_crossed: bool,
        is_offset: bool,
    },
    Value(u8),
    None,
}

impl AddressValue {
    fn addr(addr: u16) -> Self {
        Self::Address {
            addr,
            page_boundary_crossed: false,
            is_offset: false,
        }
    }

    fn offset_addr(addr: u16, page_boundary_crossed: bool) -> Self {
        Self::Address {
            addr,
            page_boundary_crossed,
            is_offset: true,
        }
    }
}

impl AddressMode {
    pub fn execute(&self, system: &mut System, clocks: &mut usize) -> AddressValue {
        match self {
            Self::Absolute => {
                *clocks += 3;
                let low = system.next_byte() as u16;
                let high = system.next_byte() as u16;
                AddressValue::addr((high << 8) + low)
            }
            Self::AbsoluteX | Self::AbsoluteY => {
                *clocks += 3;
                let offset = match self {
                    Self::AbsoluteX => system.chip.x,
                    Self::AbsoluteY => system.chip.y,
                    _ => unreachable!(),
                } as i8;
                let low = system.next_byte() as u16;
                let high = system.next_byte() as u16;
                let addr = (high << 8) + low;
                let offset_addr = addr.wrapping_add(offset as u16);
                let page_boundary_crossed = addr & 0xFF00 != offset_addr & 0xFF00;
                AddressValue::offset_addr(offset_addr, page_boundary_crossed)
            }
            Self::AbsoluteI => {
                *clocks += 5;
                let low = system.next_byte() as u16;
                let high = system.next_byte() as u16;
                let addr = (high << 8) + low;
                let low_indirect = system.memory_get(addr) as u16;
                let high_indirect = system.memory_get(addr + 1) as u16;
                AddressValue::addr((high_indirect << 8) + low_indirect)
            }
            Self::Immediate => {
                *clocks += 1;
                AddressValue::Value(system.next_byte())
            }
            Self::Relative => {
                let arg = system.next_byte() as i8;
                AddressValue::addr(system.chip.pc.wrapping_add(arg as u16))
            }
            Self::ZeroPage => {
                *clocks += 2;
                AddressValue::addr(system.next_byte() as u16)
            }
            // TODO: wrap-around
            Self::ZeroPageIX => {
                *clocks += 5;
                let addr = (system.next_byte() + system.chip.x) as u16;
                let low_indirect = system.memory_get(addr) as u16;
                let high_indirect = system.memory_get(addr + 1) as u16;
                AddressValue::addr((high_indirect << 8) + low_indirect)
            }
            // TODO: wrap-around
            Self::ZeroPageY => {
                *clocks += 3;
                AddressValue::addr((system.next_byte() + system.chip.y) as u16)
            }
            // TODO: wrap-around
            Self::ZeroPageX => {
                *clocks += 3;
                AddressValue::addr((system.next_byte() + system.chip.x) as u16)
            }
            // TODO: wrap-around
            Self::ZeroPageIY => {
                *clocks += 4;
                let offset = system.chip.y as i8;
                let addr = system.next_byte() as u16;

                let low_indirect = system.memory_get(addr) as u16;
                let high_indirect = system.memory_get(addr + 1) as u16;
                let addr = (high_indirect << 8) + low_indirect;

                let offset_addr = addr.wrapping_add(offset as u16);
                let page_boundary_crossed = addr & 0xFF00 != offset_addr & 0xFF00;

                AddressValue::offset_addr(offset_addr, page_boundary_crossed)
            }
            _ => AddressValue::None,
        }
    }
}

impl TryFrom<u8> for Instruction {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use AddressMode::*;
        use Instruction::*;

        Ok(match value {
            0x69 => Adc(Immediate),
            0x65 => Adc(ZeroPage),
            0x75 => Adc(ZeroPageX),
            0x6D => Adc(Absolute),
            0x7D => Adc(AbsoluteX),
            0x79 => Adc(AbsoluteY),
            0x61 => Adc(ZeroPageIX),
            0x71 => Adc(ZeroPageIY),
            0x29 => And(Immediate),
            0x25 => And(ZeroPage),
            0x35 => And(ZeroPageX),
            0x2D => And(Absolute),
            0x3D => And(AbsoluteX),
            0x39 => And(AbsoluteY),
            0x21 => And(ZeroPageIX),
            0x31 => And(ZeroPageIY),
            0x0A => Asl(Accumulator),
            0x06 => Asl(ZeroPage),
            0x16 => Asl(ZeroPageX),
            0x0E => Asl(Absolute),
            0x1E => Asl(AbsoluteX),
            0x24 => Bit(ZeroPage),
            0x2C => Bit(Absolute),
            0x10 => Bpl(Relative),
            0x30 => Bmi(Relative),
            0x50 => Bvc(Relative),
            0x70 => Bvs(Relative),
            0x90 => Bcc(Relative),
            0xB0 => Bcs(Relative),
            0xD0 => Bne(Relative),
            0xF0 => Beq(Relative),
            0x00 => Brk(Implied),
            0xc9 => Cmp(Immediate),
            0xc5 => Cmp(ZeroPage),
            0xD5 => Cmp(ZeroPageX),
            0xCD => Cmp(Absolute),
            0xDD => Cmp(AbsoluteX),
            0xD9 => Cmp(AbsoluteY),
            0xC1 => Cmp(ZeroPageIX),
            0xD1 => Cmp(ZeroPageIY),
            0xE0 => Cpx(Immediate),
            0xE4 => Cpx(ZeroPage),
            0xEC => Cpx(Absolute),
            0xC0 => Cpy(Immediate),
            0xC4 => Cpy(ZeroPage),
            0xCC => Cpy(Absolute),
            0xC6 => Dec(ZeroPage),
            0xD6 => Dec(ZeroPageX),
            0xCE => Dec(Absolute),
            0xDE => Dec(AbsoluteX),
            0x49 => Eor(Immediate),
            0x45 => Eor(ZeroPage),
            0x55 => Eor(ZeroPageX),
            0x4D => Eor(Absolute),
            0x5D => Eor(AbsoluteX),
            0x59 => Eor(AbsoluteY),
            0x41 => Eor(ZeroPageIX),
            0x51 => Eor(ZeroPageIY),
            0x18 => Clc(Implied),
            0x38 => Sec(Implied),
            0x58 => Cli(Implied),
            0x78 => Sei(Implied),
            0xB8 => Clv(Implied),
            0xD8 => Cld(Implied),
            0xF8 => Sed(Implied),
            0xE6 => Inc(ZeroPage),
            0xF6 => Inc(ZeroPageX),
            0xEE => Inc(Absolute),
            0xFE => Inc(AbsoluteX),
            0x4C => Jmp(Absolute),
            0x6C => Jmp(AbsoluteI),
            0x20 => Jsr(Absolute),
            0xA9 => Lda(Immediate),
            0xA5 => Lda(ZeroPage),
            0xB5 => Lda(ZeroPageX),
            0xAD => Lda(Absolute),
            0xBD => Lda(AbsoluteX),
            0xB9 => Lda(AbsoluteY),
            0xA1 => Lda(ZeroPageIX),
            0xB1 => Lda(ZeroPageIY),
            0xA2 => Ldx(Immediate),
            0xA6 => Ldx(ZeroPage),
            0xB6 => Ldx(ZeroPageY),
            0xAE => Ldx(Absolute),
            0xBE => Ldx(AbsoluteY),
            0xA0 => Ldy(Immediate),
            0xA4 => Ldy(ZeroPage),
            0xB4 => Ldy(ZeroPageX),
            0xAC => Ldy(Absolute),
            0xBC => Ldy(AbsoluteX),
            0x4A => Lsr(Accumulator),
            0x46 => Lsr(ZeroPage),
            0x56 => Lsr(ZeroPageX),
            0x4E => Lsr(Absolute),
            0x5E => Lsr(AbsoluteX),
            0xEA => Nop(Implied),
            0x09 => Ora(Immediate),
            0x05 => Ora(ZeroPage),
            0x15 => Ora(ZeroPageX),
            0x0D => Ora(Absolute),
            0x1D => Ora(AbsoluteX),
            0x19 => Ora(AbsoluteY),
            0x01 => Ora(ZeroPageIX),
            0x11 => Ora(ZeroPageIY),
            0xAA => Tax(Implied),
            0x8A => Txa(Implied),
            0xCA => Dex(Implied),
            0xE8 => Inx(Implied),
            0xA8 => Tay(Implied),
            0x98 => Tya(Implied),
            0x88 => Dey(Implied),
            0xC8 => Iny(Implied),
            0x2A => Rol(Accumulator),
            0x26 => Rol(ZeroPage),
            0x36 => Rol(ZeroPageX),
            0x2E => Rol(Absolute),
            0x3E => Rol(AbsoluteX),
            0x6A => Ror(Accumulator),
            0x66 => Ror(ZeroPage),
            0x76 => Ror(ZeroPageX),
            0x6E => Ror(Absolute),
            0x7E => Ror(AbsoluteX),
            0x40 => Rti(Implied),
            0x60 => Rts(Implied),
            0xE9 => Sbc(Immediate),
            0xE5 => Sbc(ZeroPage),
            0xF5 => Sbc(ZeroPageX),
            0xED => Sbc(Absolute),
            0xFD => Sbc(AbsoluteX),
            0xF9 => Sbc(AbsoluteY),
            0xE1 => Sbc(ZeroPageIX),
            0xF1 => Sbc(ZeroPageIY),
            0x85 => Sta(ZeroPage),
            0x95 => Sta(ZeroPageX),
            0x8D => Sta(Absolute),
            0x9D => Sta(AbsoluteX),
            0x99 => Sta(AbsoluteY),
            0x81 => Sta(ZeroPageIX),
            0x91 => Sta(ZeroPageIY),
            0x9A => Txs(Implied),
            0xBA => Tsx(Implied),
            0x48 => Pha(Implied),
            0x68 => Pla(Implied),
            0x08 => Php(Implied),
            0x28 => Plp(Implied),
            0x86 => Stx(ZeroPage),
            0x96 => Stx(ZeroPageY),
            0x8E => Stx(Absolute),
            0x84 => Sty(ZeroPage),
            0x94 => Sty(ZeroPageY),
            0x8C => Sty(Absolute),
            // Illegal opcodes
            0x04 => Dop(ZeroPage),
            _ => return Err(format!("Unknown instruction: {:02X}", value)),
        })
    }
}

impl Instruction {}

#[cfg(test)]
mod test {
    use super::AddressMode::*;
    use super::Instruction::*;
    use super::*;
    use crate::System;

    #[test]
    fn adds_cycle_with_page_boundary_cross() {
        let mut system = System::new([0u8; 4096]);
        system.program[0] = 0xFF;
        system.program[1] = 0x10;
        system.chip.x = 0x01;

        let clocks = Adc(AbsoluteX).execute(&mut system).unwrap();
        assert_eq!(clocks, 5);

        system.program[0] = 0x00;
        system.program[1] = 0x11;
        system.chip.pc = 0x1000;
        system.chip.y = 0xFF;

        let clocks = Adc(AbsoluteY).execute(&mut system).unwrap();
        assert_eq!(clocks, 5);

        system.program[0] = 0x80;
        system.memory[0] = 0x00;
        system.memory[1] = 0x11;
        system.chip.pc = 0x1000;
        system.chip.y = 0xFF;
        let clocks = Adc(ZeroPageIY).execute(&mut system).unwrap();
        assert_eq!(clocks, 6);
    }

    #[test]
    fn without_page_boundary_cross() {
        let mut system = System::new([0u8; 4096]);
        system.program[0] = 0x00;
        system.program[1] = 0x80;
        system.chip.x = 0x01;

        let clocks = Sta(AbsoluteX).execute(&mut system).unwrap();
        assert_eq!(clocks, 5);
    }

    #[test]
    fn all_instructions_with_page_boundary() {
        let mut system = System::new([0u8; 4096]);

        // Adc
        system.program[0] = 0x00;
        system.program[1] = 0x11;

        let instruction = Adc(AbsoluteY);
        system.chip.y = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 4);

        system.chip.y = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 5);

        // Sbc
        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x11;

        let instruction = Sbc(AbsoluteY);
        system.chip.y = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 4);

        system.chip.y = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 5);

        // And
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0x00;

        let instruction = And(AbsoluteY);
        system.chip.y = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 4);

        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x02;

        system.chip.y = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 5);

        // Cmp
        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x11;

        let instruction = Cmp(AbsoluteY);
        system.chip.y = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 4);

        system.chip.y = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 5);

        // Eor
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0x00;

        let instruction = Eor(AbsoluteY);
        system.chip.y = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 4);

        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x02;

        system.chip.y = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 5);

        // Lda
        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x11;

        let instruction = Lda(AbsoluteY);
        system.chip.y = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 4);

        system.chip.y = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 5);

        // Ldx
        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x11;

        let instruction = Ldx(AbsoluteY);
        system.chip.y = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 4);

        system.chip.y = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 5);

        // Ldy
        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x11;

        let instruction = Ldy(AbsoluteX);
        system.chip.x = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 4);

        system.chip.x = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 5);

        // Ora
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0x00;

        let instruction = Ora(AbsoluteY);
        system.chip.y = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 4);

        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x02;

        system.chip.y = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 5);
    }

    #[test]
    fn all_instructions_without_page_boundary() {
        let mut system = System::new([0u8; 4096]);

        // ASL
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0x00;

        let instruction = Asl(AbsoluteX);
        system.chip.x = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x01;

        system.chip.x = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        // Dec
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0x00;

        let instruction = Dec(AbsoluteX);
        system.chip.x = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x01;

        system.chip.x = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        // Inc
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0x00;

        let instruction = Inc(AbsoluteX);
        system.chip.x = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x01;

        system.chip.x = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        // Lsr
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0x00;

        let instruction = Lsr(AbsoluteX);
        system.chip.x = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x01;

        system.chip.x = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        // Rol
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0x00;

        let instruction = Rol(AbsoluteX);
        system.chip.x = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x01;

        system.chip.x = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        // Ror
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0x00;

        let instruction = Ror(AbsoluteX);
        system.chip.x = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x01;

        system.chip.x = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 7);

        // Sta
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0x00;

        let instruction = Sta(AbsoluteX);
        system.chip.x = 0x01;
        assert_eq!(instruction.execute(&mut system).unwrap(), 5);

        system.chip.pc = 0x1000;
        system.program[0] = 0x00;
        system.program[1] = 0x01;

        let instruction = Sta(AbsoluteY);
        system.chip.y = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 5);

        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0x00;
        system.memory[0] = 0x00;
        system.memory[1] = 0x01;

        let instruction = Sta(ZeroPageIY);
        system.chip.y = 0xFF;
        assert_eq!(instruction.execute(&mut system).unwrap(), 6);
    }

    #[test]
    fn test_address_mode_absolute_execute() {
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;

        system.program[0] = 0xFF;
        system.program[1] = 0x00;
        assert_eq!(
            AddressMode::Absolute.execute(&mut system, &mut clocks),
            AddressValue::addr(0x00FF)
        );
        assert_eq!(clocks, 3);
        assert_eq!(system.chip.pc, pc + 2);
    }

    #[test]
    fn test_address_mode_absolute_x_execute() {
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;
        system.chip.x = 1;

        system.program[0] = 0xFF;
        system.program[1] = 0x00;
        assert_eq!(
            AddressMode::AbsoluteX.execute(&mut system, &mut clocks),
            AddressValue::offset_addr(0x0100, true)
        );
        assert_eq!(clocks, 3);
        assert_eq!(system.chip.pc, pc + 2);
    }

    #[test]
    fn test_address_mode_absolute_y_execute() {
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;
        system.chip.y = 1;

        system.program[0] = 0xFE;
        system.program[1] = 0xFF;
        assert_eq!(
            AddressMode::AbsoluteY.execute(&mut system, &mut clocks),
            AddressValue::offset_addr(0xFFFF, false)
        );
        assert_eq!(clocks, 3);
        assert_eq!(system.chip.pc, pc + 2);
    }

    #[test]
    fn test_address_mode_absolute_i_execute() {
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;

        system.program[0] = 128;
        system.program[1] = 0x00;
        system.memory[0] = 0x11;
        system.memory[1] = 0xE8;
        assert_eq!(
            AddressMode::AbsoluteI.execute(&mut system, &mut clocks),
            AddressValue::addr(0xE811)
        );
        assert_eq!(clocks, 5);
        assert_eq!(system.chip.pc, pc + 2);
    }

    #[test]
    fn test_address_mode_accumulator_execute() {
        // TODO: accumulator mode seems to be faster than other modes. It's only used with shifts
        // and rotates. The normal math won't work
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;

        assert_eq!(
            AddressMode::Accumulator.execute(&mut system, &mut clocks),
            AddressValue::None
        );
        assert_eq!(clocks, 0);
        assert_eq!(system.chip.pc, pc);
    }

    #[test]
    fn test_address_mode_immediate_execute() {
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;

        system.program[0] = 0x76;
        assert_eq!(
            AddressMode::Immediate.execute(&mut system, &mut clocks),
            AddressValue::Value(0x76)
        );
        assert_eq!(clocks, 1);
        assert_eq!(system.chip.pc, pc + 1);
    }

    #[test]
    fn test_address_mode_implied_execute() {
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;

        assert_eq!(
            AddressMode::Implied.execute(&mut system, &mut clocks),
            AddressValue::None
        );
        assert_eq!(clocks, 0);
        assert_eq!(system.chip.pc, pc);
    }

    #[test]
    fn test_address_mode_relative_execute() {
        // TODO: All clock calculations will have to be done in the instruction itself since the
        // state of the chip is required to determine if clocks are added
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;

        system.program[0] = 0x76;
        system.program[1] = 0xFE;
        // Address with offset from end of instruction
        assert_eq!(
            AddressMode::Relative.execute(&mut system, &mut clocks),
            AddressValue::addr(0x1077)
        );
        assert_eq!(clocks, 0);
        assert_eq!(system.chip.pc, pc + 1);

        assert_eq!(
            AddressMode::Relative.execute(&mut system, &mut clocks),
            AddressValue::addr(0x1000)
        );
        assert_eq!(clocks, 0);
        assert_eq!(system.chip.pc, pc + 2);
    }

    #[test]
    fn test_address_mode_zero_page_execute() {
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;

        system.program[0] = 0x76;
        assert_eq!(
            AddressMode::ZeroPage.execute(&mut system, &mut clocks),
            AddressValue::addr(0x0076)
        );
        assert_eq!(clocks, 2);
        assert_eq!(system.chip.pc, pc + 1);
    }

    #[test]
    fn test_address_mode_zero_page_ix_execute() {
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;
        system.chip.x = 1;

        system.program[0] = 128;
        system.memory[1] = 0xEF;
        system.memory[2] = 0xBE;
        assert_eq!(
            AddressMode::ZeroPageIX.execute(&mut system, &mut clocks),
            AddressValue::addr(0xBEEF)
        );
        assert_eq!(clocks, 5);
        assert_eq!(system.chip.pc, pc + 1);
    }

    #[test]
    fn test_address_mode_zero_page_y_execute() {
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;
        system.chip.y = 0x10;

        system.program[0] = 0x00;
        assert_eq!(
            AddressMode::ZeroPageY.execute(&mut system, &mut clocks),
            AddressValue::addr(0x0010)
        );
        assert_eq!(clocks, 3);
        assert_eq!(system.chip.pc, pc + 1);
    }

    #[test]
    fn test_address_mode_zero_page_x_execute() {
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;
        system.chip.x = 0x10;

        system.program[0] = 0x00;
        assert_eq!(
            AddressMode::ZeroPageX.execute(&mut system, &mut clocks),
            AddressValue::addr(0x0010)
        );
        assert_eq!(clocks, 3);
        assert_eq!(system.chip.pc, pc + 1);
    }

    #[test]
    fn test_address_mode_zero_page_iy_execute() {
        let mut system = System::new([0u8; 4096]);
        let mut clocks = 0;
        let pc = system.chip.pc;
        system.chip.y = 0x10;

        system.program[0] = 128;
        system.memory[0] = 0xEF;
        system.memory[1] = 0xBE;
        assert_eq!(
            AddressMode::ZeroPageIY.execute(&mut system, &mut clocks),
            AddressValue::offset_addr(0xBEEF + 0x10, false)
        );
        assert_eq!(clocks, 4);
        assert_eq!(system.chip.pc, pc + 1);
    }

    #[test]
    fn test_instruction_type_adc_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.a = 127;
        system.chip.pc = 0x1000;
        system.program[0] = 1;
        let clocks = Adc(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 128);
        assert_eq!(clocks, 2);

        assert!(!system.chip.c);
        assert!(system.chip.v);
        assert!(system.chip.n);
        assert!(!system.chip.z);

        // Adds with carry
        system.chip.a = 0;
        system.chip.c = true;
        system.chip.pc = 0x1000;
        system.program[0] = 1;
        Adc(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 2);

        // Carry and overflow
        system.chip.a = 0x80;
        system.chip.c = false;
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        Adc(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0);
        assert!(system.chip.c);
        assert!(system.chip.v);

        // From address
        system.chip.a = 0x80;
        system.chip.c = false;
        system.chip.pc = 0x1000;
        system.program[0] = 0x05;
        system.program[1] = 0x10;
        system.program[5] = 0x80;
        Adc(Absolute).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0);
        assert!(system.chip.c);
        assert!(system.chip.v);
    }

    #[test]
    fn test_instruction_type_sbc_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.a = 0;
        system.chip.c = true;
        system.chip.pc = 0x1000;
        system.program[0] = 1;
        let clocks = Sbc(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0xFF);
        assert_eq!(clocks, 2);

        assert!(system.chip.c);
        assert!(!system.chip.v);
        assert!(system.chip.n);
        assert!(!system.chip.z);

        // Adds with carry
        system.chip.a = 0;
        system.chip.c = false;
        system.chip.pc = 0x1000;
        system.program[0] = 1;
        Sbc(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0xFE);

        // Carry and overflow
        system.chip.a = 0x80;
        system.chip.c = false;
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        Sbc(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0xFF);
        assert!(system.chip.c);

        // From address
        system.chip.a = 208;
        system.chip.c = true;
        system.chip.pc = 0x1000;
        system.program[0] = 0x05;
        system.program[1] = 0x10;
        system.program[5] = 112;
        Sbc(Absolute).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 96);
        assert!(!system.chip.c);
        assert!(system.chip.v);
    }

    #[test]
    fn test_instruction_type_and_execute() {
        let mut system = System::new([0u8; 4096]);

        // And with immediate value
        system.chip.a = 0b1010_1010;
        system.chip.pc = 0x1000;
        system.program[0] = 0b0101_0101;
        let clocks = And(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0);
        assert_eq!(clocks, 2);
        assert!(!system.chip.n);
        assert!(system.chip.z);

        // And with address
        system.chip.a = 0b1010_1010;
        system.chip.pc = 0x1000;
        system.program[0] = 0x8C;
        system.memory[0xC] = 0b1111_0000;
        And(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0b1010_0000);
        assert!(system.chip.n);
        assert!(!system.chip.z);
    }

    #[test]
    fn test_instruction_type_ora_execute() {
        let mut system = System::new([0u8; 4096]);

        // And with immediate value
        system.chip.a = 0b0000_0000;
        system.chip.pc = 0x1000;
        system.program[0] = 0b0000_0000;
        let clocks = Ora(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0);
        assert_eq!(clocks, 2);
        assert!(!system.chip.n);
        assert!(system.chip.z);

        // And with address
        system.chip.a = 0b1010_1010;
        system.memory[0xC] = 0b1111_0000;
        system.chip.pc = 0x1000;
        system.program[0] = 0x8C;
        Ora(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0b1111_1010);
        assert!(system.chip.n);
        assert!(!system.chip.z);
    }

    // Asl,
    #[test]
    fn test_instruction_type_asl_execute() {
        let mut system = System::new([0u8; 4096]);

        // ASL accumulator
        // 2 clocks
        system.chip.a = 0b1010_1010;
        let clocks = Asl(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0b0101_0100);
        assert_eq!(clocks, 2);
        assert!(!system.chip.n);
        assert!(!system.chip.z);
        assert!(system.chip.c);

        // ASL memory address
        // 3 clocks
        system.chip.pc = 0x1000;
        system.program[0] = 0x90;
        system.memory[0x10] = 0b1000_0000;
        let clocks = Asl(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.memory_get(0x10 + 0x80), 0);
        assert_eq!(clocks, 5);
        assert!(!system.chip.n);
        assert!(system.chip.z);
        assert!(system.chip.c);
    }

    // Bit,
    #[test]
    fn test_instruction_type_bit_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.a = 0b1010_1010;
        system.chip.pc = 0x1000;
        system.program[0] = 0x1A;
        system.program[1] = 0x15;
        system.program[0x051A] = 0b1100_0000;
        let clocks = Bit(Absolute).execute(&mut system).unwrap();
        assert_eq!(clocks, 4);
        assert!(system.chip.n);
        assert!(!system.chip.z);
        assert!(!system.chip.v);

        system.chip.a = 0b0111_1111;
        system.chip.pc = 0x1000;
        system.program[0] = 0x90;
        system.memory[0x10] = 0b1100_0000;
        let clocks = Bit(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(clocks, 3);
        assert!(!system.chip.n);
        assert!(!system.chip.z);
        assert!(system.chip.v);
    }

    #[test]
    fn test_instruction_type_bpl_execute() {
        let mut system = System::new([0u8; 4096]);

        // Not Taken
        system.chip.n = true;
        system.chip.pc = 0x1000;
        system.program[0] = 0x99;
        // Max on the same page
        let clocks = Bpl(Relative).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert_eq!(system.chip.pc, 0x1001);

        // Taken
        system.chip.n = false;
        system.chip.pc = 0x1000;
        system.program[0] = 0x7F;
        // Max on the same page
        let clocks = Bpl(Relative).execute(&mut system).unwrap();
        assert_eq!(clocks, 3);
        assert_eq!(system.chip.pc, 0x1080);

        // Taken -- Across page
        system.chip.n = false;
        system.chip.pc = 0x10FE;
        system.program[0xFE] = 1;
        // Max on the same page
        let clocks = Bpl(Relative).execute(&mut system).unwrap();
        assert_eq!(clocks, 4);
        assert_eq!(system.chip.pc, 0x1100);
    }

    #[test]
    fn test_instruction_type_bmi_execute() {
        let mut system = System::new([0u8; 4096]);

        // Not Taken
        system.chip.n = false;
        system.chip.pc = 0x1000;
        system.program[0] = 0x99;
        // Max on the same page
        let clocks = Bmi(Relative).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert_eq!(system.chip.pc, 0x1001);

        // Taken
        system.chip.n = true;
        system.chip.pc = 0x1000;
        system.program[0] = 0x7F;
        // Max on the same page
        let clocks = Bmi(Relative).execute(&mut system).unwrap();
        assert_eq!(clocks, 3);
        assert_eq!(system.chip.pc, 0x1080);
    }

    #[test]
    fn test_instruction_type_bvc_execute() {
        let mut system = System::new([0u8; 4096]);

        // Not Taken
        system.chip.v = true;
        // Max on the same page
        system.chip.pc = 0x1000;
        system.program[0] = 0x99;
        // Max on the same page
        let clocks = Bvc(Relative).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert_eq!(system.chip.pc, 0x1001);

        // Taken
        system.chip.v = false;
        // Max on the same page
        system.chip.pc = 0x1000;
        system.program[0] = 0x7F;
        let clocks = Bvc(Relative).execute(&mut system).unwrap();
        assert_eq!(clocks, 3);
        assert_eq!(system.chip.pc, 0x1080);
    }

    #[test]
    fn test_instruction_type_bcc_execute() {
        let mut system = System::new([0u8; 4096]);

        // Not Taken
        system.chip.c = true;
        system.chip.pc = 0x1000;
        system.program[0] = 0x99;
        let clocks = Bcc(Relative).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert_eq!(system.chip.pc, 0x1001);
    }

    #[test]
    fn test_instruction_type_bcs_execute() {
        let mut system = System::new([0u8; 4096]);

        // Not Taken
        system.chip.c = false;
        system.chip.pc = 0x1000;
        system.program[0] = 0x99;
        let clocks = Bcs(Relative).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert_eq!(system.chip.pc, 0x1001);
    }

    #[test]
    fn test_instruction_type_bne_execute() {
        let mut system = System::new([0u8; 4096]);

        // Not Taken
        system.chip.z = true;
        system.chip.pc = 0x1000;
        system.program[0] = 0x99;
        let clocks = Bne(Relative).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert_eq!(system.chip.pc, 0x1001);
    }

    #[test]
    fn test_instruction_type_beq_execute() {
        let mut system = System::new([0u8; 4096]);

        // Not Taken
        system.chip.z = false;
        system.chip.pc = 0x1000;
        system.program[0] = 0x99;
        let clocks = Beq(Relative).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert_eq!(system.chip.pc, 0x1001);
    }

    #[test]
    #[should_panic(expected = "BRK and RTI not implemented -- save for a fun stream topic")]
    fn test_instruction_type_brk_execute() {
        let mut system = System::new([0u8; 4096]);

        Instruction::Brk(Implied).execute(&mut system).unwrap();
    }

    #[test]
    fn test_instruction_type_cmp_execute() {
        let mut system = System::new([0u8; 4096]);

        // equal -- Immediate
        system.chip.a = 0x01;
        system.chip.pc = 0x1000;
        system.program[0] = 0x01;
        let clocks = Cmp(Immediate).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);
        assert!(!system.chip.c);

        // greater -- Immediate
        system.chip.a = 0x01;
        system.chip.pc = 0x1000;
        system.program[0] = 0x02;
        let clocks = Cmp(Immediate).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert!(!system.chip.z);
        assert!(system.chip.n);
        assert!(!system.chip.c);

        // less -- Immediate
        system.chip.a = 0x02;
        system.chip.pc = 0x1000;
        system.program[0] = 0x01;
        let clocks = Cmp(Immediate).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert!(!system.chip.z);
        assert!(!system.chip.n);
        assert!(system.chip.c);

        // equal -- Address
        system.chip.a = 0x01;
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.memory[0] = 0x01;
        let clocks = Cmp(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(clocks, 3);
        assert!(system.chip.z);
        assert!(!system.chip.n);
        assert!(!system.chip.c);
    }

    #[test]
    fn test_instruction_type_cpx_execute() {
        let mut system = System::new([0u8; 4096]);

        // equal -- Immediate
        system.chip.x = 0x01;
        system.chip.pc = 0x1000;
        system.program[0] = 0x01;
        let clocks = Cpx(Immediate).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);
        assert!(!system.chip.c);
    }

    #[test]
    fn test_instruction_type_cpy_execute() {
        let mut system = System::new([0u8; 4096]);

        // equal -- Immediate
        system.chip.y = 0x01;
        system.chip.pc = 0x1000;
        system.program[0] = 0x01;
        let clocks = Cpy(Immediate).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);
        assert!(!system.chip.c);
    }

    #[test]
    fn test_instruction_type_dec_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Address
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.memory[0] = 0x01;
        let clocks = Dec(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.memory[0], 0x00);
        assert_eq!(clocks, 5);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // negative -- Address
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.memory[0] = 0x00;
        Dec(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.memory[0], 0xFF);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_eor_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Immediate
        system.chip.a = 0xFF;
        system.chip.pc = 0x1000;
        system.program[0] = 0xFF;
        let clocks = Eor(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // negative -- Immediate
        system.chip.a = 0xFF;
        system.chip.pc = 0x1000;
        system.program[0] = 0x7F;
        Eor(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0x80);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_clc_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.c = true;
        let clocks = Clc(Implied).execute(&mut system).unwrap();
        assert!(!system.chip.c);
        assert_eq!(clocks, 2);
    }

    #[test]
    fn test_instruction_type_sec_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.c = false;
        let clocks = Sec(Implied).execute(&mut system).unwrap();
        assert!(system.chip.c);
        assert_eq!(clocks, 2);
    }

    #[test]
    fn test_instruction_type_cli_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.i = true;
        let clocks = Cli(Implied).execute(&mut system).unwrap();
        assert!(!system.chip.i);
        assert_eq!(clocks, 2);
    }

    #[test]
    fn test_instruction_type_sei_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.i = false;
        let clocks = Sei(Implied).execute(&mut system).unwrap();
        assert!(system.chip.i);
        assert_eq!(clocks, 2);
    }

    #[test]
    fn test_instruction_type_clv_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.v = true;
        let clocks = Clv(Implied).execute(&mut system).unwrap();
        assert!(!system.chip.v);
        assert_eq!(clocks, 2);
    }

    #[test]
    fn test_instruction_type_cld_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.d = true;
        let clocks = Cld(Implied).execute(&mut system).unwrap();
        assert!(!system.chip.d);
        assert_eq!(clocks, 2);
    }

    #[test]
    fn test_instruction_type_sed_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.d = false;
        let clocks = Sed(Implied).execute(&mut system).unwrap();
        assert!(system.chip.d);
        assert_eq!(clocks, 2);
    }

    #[test]
    fn test_instruction_type_inc_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Address
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.memory[0] = 0xFF;
        let clocks = Inc(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.memory[0], 0x00);
        assert_eq!(clocks, 5);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // negative -- Address
        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.memory[0] = 0x80;
        let clocks = Inc(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.memory[0], 0x81);
        assert_eq!(clocks, 5);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_jmp_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.pc = 0x1000;
        system.program[0] = 0x80;
        system.program[1] = 0xFF;
        let clocks = Jmp(Absolute).execute(&mut system).unwrap();
        assert_eq!(system.chip.pc, 0xFF80);
        assert_eq!(clocks, 3);
    }

    #[test]
    fn test_instruction_type_jsr_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.pc = 0x1EAD;
        system.program[0x0EAD] = 0x80;
        system.program[0x0EAD + 1] = 0xFF;
        system.chip.sp = 0xFF;
        let clocks = Jsr(Absolute).execute(&mut system).unwrap();
        assert_eq!(system.chip.pc, 0xFF80);
        assert_eq!(system.chip.sp, 0xFD);
        assert_eq!(system.memory[0x7F], 0x1E);
        assert_eq!(system.memory[0x7E], 0xAE);
        assert_eq!(clocks, 6);
    }

    #[test]
    fn test_instruction_type_lda_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Immediate
        system.chip.a = 0xFF;
        system.chip.pc = 0x1000;
        system.program[0] = 0;
        let clocks = Lda(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // negative -- Address
        system.chip.a = 0x00;
        system.chip.pc = 0x1000;
        system.program[0] = 0x02;
        system.program[1] = 0x10;
        system.program[2] = 0xFF;
        let clocks = Lda(Absolute).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0xFF);
        assert_eq!(clocks, 4);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_ldx_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Immediate
        system.chip.x = 0xFF;
        system.chip.pc = 0x1000;
        system.program[0] = 0;
        let clocks = Ldx(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.x, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);
    }

    #[test]
    fn test_instruction_type_ldy_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Immediate
        system.chip.y = 0xFF;
        system.chip.pc = 0x1000;
        system.program[0] = 0;
        let clocks = Ldy(Immediate).execute(&mut system).unwrap();
        assert_eq!(system.chip.y, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);
    }

    #[test]
    fn test_instruction_type_lsr_execute() {
        let mut system = System::new([0u8; 4096]);

        // LSR accumulator
        // 2 clocks
        system.chip.a = 0b1010_1010;
        let clocks = Lsr(Accumulator).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0b0101_0101);
        assert_eq!(clocks, 2);
        assert!(!system.chip.n);
        assert!(!system.chip.z);
        assert!(!system.chip.c);

        // ASL memory address
        // 3 clocks
        system.memory[0x10] = 0b0000_0001;
        system.chip.pc = 0x1000;
        system.program[0] = 0x90;
        let clocks = Lsr(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.memory_get(0x10 + 0x80), 0);
        assert_eq!(clocks, 5);
        assert!(!system.chip.n);
        assert!(system.chip.z);
        assert!(system.chip.c);
    }

    #[test]
    fn test_instruction_type_nop_execute() {
        let mut system = System::new([0u8; 4096]);

        let clocks = Nop(Implied).execute(&mut system).unwrap();
        assert_eq!(clocks, 2);
    }

    #[test]
    fn test_instruction_type_tax_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Implied
        system.chip.x = 0xFF;
        system.chip.a = 0x00;

        let clocks = Tax(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.x, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // zero -- Implied
        system.chip.a = 0xFF;
        system.chip.x = 0x00;

        let clocks = Tax(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.x, 0xFF);
        assert_eq!(clocks, 2);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_txa_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Implied
        system.chip.a = 0xFF;
        system.chip.x = 0x00;

        let clocks = Txa(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // zero -- Implied
        system.chip.x = 0xFF;
        system.chip.a = 0x00;

        let clocks = Txa(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0xFF);
        assert_eq!(clocks, 2);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_tay_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Implied
        system.chip.y = 0xFF;
        system.chip.a = 0x00;

        let clocks = Tay(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.y, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // zero -- Implied
        system.chip.a = 0xFF;
        system.chip.y = 0x00;

        let clocks = Tay(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.y, 0xFF);
        assert_eq!(clocks, 2);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_tya_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Implied
        system.chip.a = 0xFF;
        system.chip.y = 0x00;

        let clocks = Tya(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // zero -- Implied
        system.chip.y = 0xFF;
        system.chip.a = 0x00;

        let clocks = Tya(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0xFF);
        assert_eq!(clocks, 2);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_dex_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Implied
        system.chip.x = 0x01;

        let clocks = Dex(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.x, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // zero -- Implied
        system.chip.x = 0x00;

        let clocks = Dex(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.x, 0xFF);
        assert_eq!(clocks, 2);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_dey_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Implied
        system.chip.y = 0x01;

        let clocks = Dey(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.y, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // zero -- Implied
        system.chip.y = 0x00;

        let clocks = Dey(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.y, 0xFF);
        assert_eq!(clocks, 2);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_inx_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Implied
        system.chip.x = 0xFF;

        let clocks = Inx(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.x, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // zero -- Implied
        system.chip.x = 0x80;

        let clocks = Inx(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.x, 0x81);
        assert_eq!(clocks, 2);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_iny_execute() {
        let mut system = System::new([0u8; 4096]);

        // zero -- Implied
        system.chip.y = 0xFF;

        let clocks = Iny(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.y, 0);
        assert_eq!(clocks, 2);
        assert!(system.chip.z);
        assert!(!system.chip.n);

        // zero -- Implied
        system.chip.y = 0x80;

        let clocks = Iny(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.y, 0x81);
        assert_eq!(clocks, 2);
        assert!(!system.chip.z);
        assert!(system.chip.n);
    }

    #[test]
    fn test_instruction_type_rol_execute() {
        let mut system = System::new([0u8; 4096]);

        // Rol accumulator
        // 2 clocks
        system.chip.c = false;
        system.chip.a = 0b1010_1010;
        let clocks = Rol(Accumulator).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0b0101_0100);
        assert_eq!(clocks, 2);
        assert!(!system.chip.n);
        assert!(!system.chip.z);
        assert!(system.chip.c);

        // Rol with carry accumulator
        // 2 clocks
        system.chip.a = 0b0100_0000;
        let clocks = Rol(Accumulator).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0b1000_0001);
        assert_eq!(clocks, 2);
        assert!(system.chip.n);
        assert!(!system.chip.z);
        assert!(!system.chip.c);

        // Rol memory address
        // 3 clocks
        system.chip.c = false;
        system.chip.pc = 0x1000;
        system.program[0] = 0x90;
        system.memory[0x10] = 0b1000_0000;
        let clocks = Rol(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.memory_get(0x10 + 0x80), 0);
        assert_eq!(clocks, 5);
        assert!(!system.chip.n);
        assert!(system.chip.z);
        assert!(system.chip.c);
    }

    #[test]
    fn test_instruction_type_ror_execute() {
        let mut system = System::new([0u8; 4096]);

        // Ror accumulator
        // 2 clocks
        system.chip.c = false;
        system.chip.a = 0b1010_1011;
        let clocks = Ror(Accumulator).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0b0101_0101);
        assert_eq!(clocks, 2);
        assert!(!system.chip.n);
        assert!(!system.chip.z);
        assert!(system.chip.c);

        // Ror with carry accumulator
        // 2 clocks
        system.chip.c = true;
        system.chip.a = 0b0000_0000;
        let clocks = Ror(Accumulator).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0b1000_0000);
        assert_eq!(clocks, 2);
        assert!(system.chip.n);
        assert!(!system.chip.z);
        assert!(!system.chip.c);

        // Ror memory address
        // 3 clocks
        system.chip.c = false;
        system.chip.pc = 0x1000;
        system.program[0] = 0x90;
        system.memory[0x10] = 0b0000_0001;
        let clocks = Ror(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.memory_get(0x10 + 0x80), 0);
        assert_eq!(clocks, 5);
        assert!(!system.chip.n);
        assert!(system.chip.z);
        assert!(system.chip.c);
    }

    #[test]
    #[should_panic(expected = "BRK and RTI not implemented -- save for a fun stream topic")]
    fn test_instruction_type_rti_execute() {
        let mut system = System::new([0u8; 4096]);

        Rti(Implied).execute(&mut system).unwrap();
    }

    #[test]
    fn test_instruction_type_rts_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.sp = 0xFD;
        system.memory[0x7F] = 0xDE;
        system.memory[0x7E] = 0xAC;
        let clocks = Rts(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.pc, 0xDEAD);
        assert_eq!(system.chip.sp, 0xFF);
        assert_eq!(clocks, 6);
    }

    #[test]
    fn test_instruction_type_tsx_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.sp = 0x01;

        let clocks = Tsx(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.x, 0x01);
        assert_eq!(clocks, 2);
    }

    #[test]
    fn test_instruction_type_txs_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.x = 0x01;

        let clocks = Txs(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.sp, 0x01);
        assert_eq!(clocks, 2);
    }

    #[test]
    fn test_instruction_type_pha_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.sp = 0xFF;
        system.chip.a = 0xAA;
        let clocks = Pha(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.sp, 0xFE);
        assert_eq!(system.memory[0x7F], 0xAA);
        assert_eq!(clocks, 3);
    }

    #[test]
    fn test_instruction_type_pla_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.sp = 0xFE;
        system.memory[0x7F] = 0xAA;
        let clocks = Pla(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.a, 0xAA);
        assert_eq!(system.chip.sp, 0xFF);
        assert_eq!(clocks, 4);
    }

    #[test]
    fn test_instruction_type_php_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.sp = 0xFF;
        system.chip.c = false;
        system.chip.z = true;
        system.chip.i = false;
        system.chip.d = true;
        system.chip.b = false;
        system.chip.v = false;
        system.chip.n = true;
        let clocks = Php(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.sp, 0xFE);
        assert_eq!(system.memory[0x7F], 0b1010_1010);
        assert_eq!(clocks, 3);
    }

    #[test]
    fn test_instruction_type_plp_execute() {
        let mut system = System::new([0u8; 4096]);

        system.memory[0x7F] = 0b0111_0101;
        system.chip.sp = 0xFE;
        let clocks = Plp(Implied).execute(&mut system).unwrap();
        assert_eq!(system.chip.sp, 0xFF);
        assert!(system.chip.c);
        assert!(!system.chip.z);
        assert!(system.chip.i);
        assert!(!system.chip.d);
        assert!(system.chip.b);
        assert!(system.chip.v);
        assert!(!system.chip.n);
        assert_eq!(clocks, 4);
    }

    #[test]
    fn test_instruction_type_sta_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.a = 0xFE;
        system.chip.pc = 0x1000;
        system.program[0] = 0x88;

        let clocks = Sta(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.memory_get(0x88), 0xFE);
        assert_eq!(clocks, 3);
    }

    #[test]
    fn test_instruction_type_stx_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.x = 0xFE;
        system.chip.pc = 0x1000;
        system.program[0] = 0x88;

        let clocks = Stx(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.memory_get(0x88), 0xFE);
        assert_eq!(clocks, 3);
    }

    #[test]
    fn test_instruction_type_sty_execute() {
        let mut system = System::new([0u8; 4096]);

        system.chip.y = 0xFE;
        system.chip.pc = 0x1000;
        system.program[0] = 0x88;

        let clocks = Sty(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(system.memory_get(0x88), 0xFE);
        assert_eq!(clocks, 3);
    }

    #[test]
    fn test_instruction_type_dop_execute() {
        let mut system = System::new([0u8; 4096]);

        let clocks = Dop(ZeroPage).execute(&mut system).unwrap();
        assert_eq!(clocks, 3);
    }
}
