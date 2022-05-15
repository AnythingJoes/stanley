use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::{stdout, BufRead, Write},
    time::Duration,
};

use crossterm::style::Color;
use crossterm::{
    cursor,
    event::{poll, read, Event as CTEvent, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};

use super::Result;
use crate::system::{instructions::InstructionValue, System};

pub enum BreakPointType {
    Number(u16),
    Symbol(String),
}

pub fn try_parse_breakpoint(s: &str) -> std::result::Result<BreakPointType, String> {
    Ok(match u16::from_str_radix(s, 16) {
        Ok(int) => BreakPointType::Number(int),
        Err(_) => BreakPointType::Symbol(s.to_owned()),
    })
}

pub trait Debugger {
    fn setup(
        &mut self,
        _program: [u8; 4096],
        _breakpoint: Option<BreakPointType>,
        _symbol_file: Option<String>,
    ) -> Result<()> {
        Ok(())
    }

    fn debug_loop(&mut self, _system: &System) -> Result<()> {
        Ok(())
    }

    fn teardown(&self) -> super::Result<()> {
        Ok(())
    }

    fn dump_disassembly(&mut self, _program: [u8; 4096]) {}
}

pub fn get_debugger(is_debug: bool) -> Box<dyn Debugger> {
    if is_debug {
        Box::new(ActiveDebugger::default())
    } else {
        Box::new(NullDebugger)
    }
}

pub struct NullDebugger;
impl Debugger for NullDebugger {}

#[derive(Default)]
pub struct ActiveDebugger {
    disassembly: Option<BTreeMap<u16, String>>,
    breakpoint: Option<u16>,
    symbol_map: HashMap<u16, String>,
    in_breakpoint: bool,
}

impl ActiveDebugger {
    fn disassemble(&mut self, program: [u8; 4096]) {
        let mut disassembly = BTreeMap::new();
        let mut program_iter = program.iter().enumerate().peekable();
        let mut in_data = false;

        while program_iter.peek().is_some() {
            let inst = program_iter.next().unwrap();
            let key = (inst.0 + 0x1000) as u16;
            let key_str = self
                .symbol_map
                .get(&key)
                .map(|val| format!("{val}:\r\n  "))
                .unwrap_or_else(|| "  ".to_owned());

            let inst_value: std::result::Result<InstructionValue, _> = (*inst.1).try_into();
            // TODO: This assumes that all code blocks are contiguous, they are not. This will need
            // to get fixed
            if in_data {
                let value = format!("{}{}", key_str, inst.1);
                disassembly.insert(key as u16, value);
                continue;
            }

            if let Ok(inst_value) = inst_value {
                let value = format!(
                    "{}{} {}",
                    key_str,
                    inst_value,
                    inst_value.format_arguments(&mut program_iter, &self.symbol_map, key)
                );
                disassembly.insert(key as u16, value);
            } else {
                in_data = true;
                let value = format!("{}{}", key_str, inst.1);
                disassembly.insert(key as u16, value);
            }
        }
        self.disassembly.replace(disassembly);
    }

    fn parse_symbol_file(&mut self, symbol_file: Option<String>) -> Result<()> {
        if symbol_file.is_none() {
            return Ok(());
        }
        let symbol_file = symbol_file.unwrap();
        let file = fs::read(symbol_file).map_err(|e| e.to_string())?;
        let map: HashMap<u16, String> = file
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| {
                if line.starts_with("---") {
                    return None;
                }
                let mut words = line.split_whitespace();
                let name = words.next().unwrap();
                let address = u16::from_str_radix(words.next().unwrap(), 16).unwrap() & 0x1FFF;
                Some((address, name.to_owned()))
            })
            .collect();
        self.symbol_map = map;
        Ok(())
    }
}

impl Debugger for ActiveDebugger {
    fn setup(
        &mut self,
        program: [u8; 4096],
        breakpoint: Option<BreakPointType>,
        symbol_file: Option<String>,
    ) -> super::Result<()> {
        let mut stdout = stdout();
        self.parse_symbol_file(symbol_file)?;
        self.disassemble(program);
        self.breakpoint = match breakpoint {
            Some(BreakPointType::Number(val)) => Some(val),
            // TODO Handle symbol
            Some(BreakPointType::Symbol(sym)) => self.symbol_map.iter().find_map(
                |(&key, value)| {
                    if *value == sym {
                        Some(key)
                    } else {
                        None
                    }
                },
            ),
            None => None,
        };
        execute!(stdout, terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        Ok(())
    }

    fn debug_loop(&mut self, system: &System) -> super::Result<()> {
        let mut stdout = stdout();
        queue!(
            stdout,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, 0),
        )?;

        queue!(
            stdout,
            style::SetForegroundColor(Color::White),
            Print(format!("{}", system.chip)),
            cursor::MoveToNextLine(1),
            Print(format!("{}", system)),
        )?;
        queue!(stdout, cursor::MoveToNextLine(1),)?;
        let riot = &system.riot;
        queue!(stdout, Print(format!("{} ", riot)))?;
        queue!(
            stdout,
            cursor::MoveToNextLine(1),
            Print(format!("{} ", system.tia)),
        )?;

        queue!(stdout, cursor::MoveToNextLine(1), Print("Program"),)?;

        let current_line = system.chip.pc & 0x1FFF;
        for (&key, line) in self
            .disassembly
            .as_ref()
            .unwrap()
            .range(current_line - 5..current_line + 5)
        {
            if current_line == key {
                queue!(
                    stdout,
                    style::SetForegroundColor(Color::Black),
                    style::SetBackgroundColor(Color::White)
                )?;
            }
            queue!(
                stdout,
                cursor::MoveToNextLine(1),
                Print(format!("{} ", line)),
            )?;
            if current_line == key {
                queue!(
                    stdout,
                    style::SetForegroundColor(Color::White),
                    style::SetBackgroundColor(Color::Black)
                )?;
            }
        }
        stdout.flush()?;

        if let Some(breakpoint) = self.breakpoint {
            let breakpoint = breakpoint & 0x1FFF;
            let pc = system.chip.pc & 0x1FFF;
            if breakpoint == pc || self.in_breakpoint {
                if let Ok(CTEvent::Key(KeyEvent { code, modifiers })) = read() {
                    match code {
                        KeyCode::Esc => return Err("User cancelled execution".into()),
                        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                            return Err("User cancelled execution".into());
                        }
                        KeyCode::Char('n') => self.in_breakpoint = true,
                        KeyCode::Char('c') => self.in_breakpoint = false,
                        _ => {}
                    }
                    return Ok(());
                }
            }
        }

        if let Ok(true) = poll(Duration::from_millis(10)) {
            if let Ok(CTEvent::Key(KeyEvent { code, modifiers })) = read() {
                if code == KeyCode::Esc
                    || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL))
                {
                    return Err("User cancelled execution".into());
                }
            }
        }
        Ok(())
    }

    fn teardown(&self) -> super::Result<()> {
        let mut stdout = stdout();
        execute!(
            stdout,
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        Ok(())
    }

    fn dump_disassembly(&mut self, program: [u8; 4096]) {
        self.disassemble(program);
        for line in self.disassembly.as_ref().unwrap().values() {
            println!("{line}")
        }
    }
}
