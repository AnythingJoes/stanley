use std::{
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

use crate::system::System;

pub struct Debugger {
    is_debug: bool,
}

impl Debugger {
    pub fn mode(is_debug: bool) -> Self {
        Self { is_debug }
    }

    pub fn setup(&self) -> super::Result<()> {
        if self.is_debug {
            let mut stdout = stdout();
            execute!(stdout, terminal::EnterAlternateScreen)?;
            terminal::enable_raw_mode()?;
        }
        Ok(())
    }

    pub fn debug_loop(&self, system: &System) -> super::Result<()> {
        if self.is_debug {
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
        }
        Ok(())
    }

    pub fn teardown(&self) -> super::Result<()> {
        if self.is_debug {
            let mut stdout = stdout();
            execute!(
                stdout,
                style::ResetColor,
                cursor::Show,
                terminal::LeaveAlternateScreen
            )?;
        }
        Ok(())
    }
}
