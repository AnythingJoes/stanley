use std::{
    error::Error,
    fs,
    io::{stdout, Write},
    time::{Duration, Instant},
};

use clap::Parser;
use crossterm::style::Color;
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};

mod system;
use system::instructions::*;
use system::System;

mod timer;
use timer::Timer;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    debug: bool,
}

fn set_up_terminal() -> Result<()> {
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    Ok(())
}

fn teardown_terminal() -> Result<()> {
    let mut stdout = stdout();
    execute!(
        stdout,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    Ok(())
}

fn clear_terminal() -> Result<()> {
    let mut stdout = stdout();
    queue!(
        stdout,
        style::ResetColor,
        terminal::Clear(ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0),
    )?;
    Ok(())
}

fn draw_terminal(system: &mut System) -> Result<()> {
    let mut stdout = stdout();
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

    stdout.flush()?;
    Ok(())
}

fn main() {
    let Args { debug } = Args::parse();

    if debug {
        set_up_terminal().expect("terminal could not be setup");
    }

    let byte_vec = fs::read("./tictactoe.bin").unwrap();
    let program = byte_vec
        .try_into()
        .expect("Program expected to be 4096 bytes was not");
    let mut system = System::new(program);
    let total_time = Instant::now();

    // Timing stuff
    let mut previous_clocks = 0;
    let mut timer = Timer::start();

    loop {
        if debug {
            clear_terminal().expect("couldn't clear terminal");
            if let Ok(true) = poll(Duration::from_millis(10)) {
                if let Ok(Event::Key(KeyEvent { code, modifiers })) = read() {
                    if code == KeyCode::Esc
                        || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL))
                    {
                        break;
                    }
                }
            }
        } else {
            let clocks_run = system.clocks - previous_clocks;
            if clocks_run > 10 {
                let clock_time = Duration::from_nanos((clocks_run * 837) as u64);
                timer.pause_for(clock_time);
                previous_clocks = system.clocks;
            }
        }

        let instruction = system.next_byte();

        match instruction {
            inst if inst == LdxI::CODE => system.execute(LdxI),
            inst if inst == LdaI::CODE => system.execute(LdaI),
            inst if inst == LdaZ::CODE => system.execute(LdaZ),
            inst if inst == LdaA::CODE => system.execute(LdaA),
            inst if inst == LdaAY::CODE => system.execute(LdaAY),
            inst if inst == StaZ::CODE => system.execute(StaZ),
            inst if inst == StaZX::CODE => system.execute(StaZX),
            inst if inst == StaA::CODE => system.execute(StaA),
            inst if inst == StxA::CODE => system.execute(StxA),
            inst if inst == Inx::CODE => system.execute(Inx),
            inst if inst == Dex::CODE => system.execute(Dex),
            inst if inst == Bne::CODE => system.execute(Bne),
            inst if inst == Bmi::CODE => system.execute(Bmi),
            inst if inst == Jmp::CODE => system.execute(Jmp),
            inst if inst == Txs::CODE => system.execute(Txs),
            inst if inst == Txa::CODE => system.execute(Txa),
            inst if inst == Tay::CODE => system.execute(Tay),
            inst if inst == Jsr::CODE => system.execute(Jsr),
            inst if inst == Rts::CODE => system.execute(Rts),
            inst if inst == Eor::CODE => system.execute(Eor),
            inst if inst == Lsr::CODE => system.execute(Lsr),
            inst if inst == Nop::CODE => system.execute(Nop),
            inst => {
                if debug {
                    std::thread::sleep(std::time::Duration::from_millis(5000));
                    break;
                }
                eprintln!("Time: {}", total_time.elapsed().as_nanos());
                eprintln!("Clocks: {}", system.clocks);
                panic!("Unknown instruction: {:02X}", inst);
            }
        }
        if debug {
            draw_terminal(&mut system).expect("couldn't draw terminal");
        }
    }
    if debug {
        teardown_terminal().expect("terminal could not be torn down");
    }
}
