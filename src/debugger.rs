use std::{
    collections::BTreeMap,
    io::{stdout, Write},
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

pub trait Debugger {
    fn setup(&mut self, _program: [u8; 4096]) -> Result<()> {
        Ok(())
    }

    fn debug_loop(&self, _system: &System) -> Result<()> {
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
}

impl ActiveDebugger {
    fn disassemble(&mut self, program: [u8; 4096]) {
        let mut disassembly = BTreeMap::new();
        let mut program_iter = program.iter().enumerate().peekable();
        let mut in_data = false;

        while program_iter.peek().is_some() {
            let inst = program_iter.next().unwrap();
            let key = inst.0 + 0x1000;
            let inst_value: std::result::Result<InstructionValue, _> = (*inst.1).try_into();
            if in_data {
                let value = format!("{key:04X}: {}", inst.1);
                disassembly.insert(key as u16, value);
                continue;
            }

            if let Ok(inst_value) = inst_value {
                let value = format!(
                    "{key:04X}: {} {}",
                    inst_value,
                    inst_value.format_arguments(&mut program_iter)
                );
                disassembly.insert(key as u16, value);
            } else {
                in_data = true;
                let value = format!("{key:04X}: {}", inst.1);
                disassembly.insert(key as u16, value);
            }
        }
        self.disassembly.replace(disassembly);
    }
}

impl Debugger for ActiveDebugger {
    fn setup(&mut self, program: [u8; 4096]) -> super::Result<()> {
        let mut stdout = stdout();
        self.disassemble(program);
        execute!(stdout, terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        Ok(())
    }

    fn debug_loop(&self, system: &System) -> super::Result<()> {
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
            .range(current_line - 10..current_line + 10)
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
