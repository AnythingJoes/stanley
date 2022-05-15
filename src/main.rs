use std::{
    error::Error,
    fs,
    io::{stdout, Write},
    time::{Duration, Instant},
};

use clap::Parser;
use crossterm::style::Color;
use crossterm::{
    cursor, execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};

mod chip;
use chip::instructions::*;
use chip::Nmos6502;

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

fn draw_terminal(chip: &Nmos6502) -> Result<()> {
    let mut stdout = stdout();
    queue!(
        stdout,
        style::SetForegroundColor(Color::White),
        cursor::MoveRight(5),
        Print("NMOS 6502"),
        cursor::MoveToNextLine(1),
        Print(format!(
            "Registers: X({:02X}) Y({:02X}) A({:02X})   |",
            chip.x, 0, chip.a
        )),
        Print(format!("PC: {:02X}   |", chip.pc)),
        Print(format!("SP: {:02X}   |", chip.sp)),
        Print(format!("CYCLES: {}", chip.cycles)),
        cursor::MoveToNextLine(1),
        cursor::MoveToNextLine(1),
        Print(format!("Flags: Z({})", chip.z)),
        cursor::MoveToNextLine(2),
        Print(format!("Next Instruction: {:02X}", chip.mmap[chip.pc])),
        cursor::MoveToNextLine(2),
        cursor::MoveRight(5),
        Print("TIA Write"),
        cursor::MoveToNextLine(1),
    )?;
    for i in 0..4 {
        for j in 0..16 {
            let tia = chip.mmap.tia[i * 16 + j];
            queue!(stdout, Print(format!("{:02X} ", tia)))?
        }

        queue!(stdout, cursor::MoveToNextLine(1))?
    }

    queue!(
        stdout,
        cursor::MoveToNextLine(2),
        cursor::MoveRight(5),
        Print("RIOT Write"),
        cursor::MoveToNextLine(1),
    )?;
    let riot = &chip.mmap.riot;
    queue!(stdout, Print(format!("{:?} ", riot)))?;

    queue!(
        stdout,
        cursor::MoveToNextLine(2),
        cursor::MoveRight(5),
        Print("RAM"),
        cursor::MoveToNextLine(1),
    )?;

    for i in 0..8 {
        for j in 0..16 {
            let memory = chip.mmap.memory[i * 16 + j];
            queue!(stdout, Print(format!("{:02X} ", memory)))?
        }

        queue!(stdout, cursor::MoveToNextLine(1))?
    }
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
    let mut chip = Nmos6502::new(program);
    let total_time = Instant::now();

    // Timing stuff
    let mut previous_cycles = 0;
    let mut timer = Timer::start();

    loop {
        if debug {
            clear_terminal().expect("couldn't clear terminal");
        } else {
            let cycles_run = chip.cycles - previous_cycles;
            if cycles_run > 10 {
                let cycle_time = Duration::from_nanos((cycles_run * 837) as u64);
                timer.pause_for(cycle_time);
                previous_cycles = chip.cycles;
            }
        }

        let instruction = chip.next_byte();

        match instruction {
            inst if inst == LdxI::CODE => chip.execute(LdxI),
            inst if inst == LdaI::CODE => chip.execute(LdaI),
            inst if inst == LdaZ::CODE => chip.execute(LdaZ),
            inst if inst == LdaA::CODE => chip.execute(LdaA),
            inst if inst == StaZ::CODE => chip.execute(StaZ),
            inst if inst == StaZX::CODE => chip.execute(StaZX),
            inst if inst == StxA::CODE => chip.execute(StxA),
            inst if inst == Inx::CODE => chip.execute(Inx),
            inst if inst == Bne::CODE => chip.execute(Bne),
            inst if inst == Bmi::CODE => chip.execute(Bmi),
            inst if inst == Txs::CODE => chip.execute(Txs),
            inst if inst == Jsr::CODE => chip.execute(Jsr),
            inst if inst == Rts::CODE => chip.execute(Rts),
            inst if inst == Eor::CODE => chip.execute(Eor),
            inst => {
                if debug {
                    std::thread::sleep(std::time::Duration::from_millis(5000));
                    teardown_terminal().expect("terminal could not be torn down");
                }
                eprintln!("Time: {}", total_time.elapsed().as_nanos());
                eprintln!("Cycles: {}", chip.cycles);
                panic!("Unknown instruction: {:02X}", inst);
            }
        }
        if debug {
            std::thread::sleep(std::time::Duration::from_millis(10));
            draw_terminal(&chip).expect("couldn't draw terminal");
        }
    }
}
