use std::{
    error::Error,
    fs,
    io::{stdout, Write},
};

use clap::Parser;
use crossterm::style::Color;
use crossterm::{
    cursor, execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};

mod chip;
use chip::Nmos6502;

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
        Print(format!("SP: {:02X}", chip.sp)),
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
            let tia = chip.mmap.tia[i * 8 + j];
            queue!(stdout, Print(format!("{:02X} ", tia)))?
        }

        queue!(stdout, cursor::MoveToNextLine(1))?
    }

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

    loop {
        if debug {
            clear_terminal().expect("couldn't clear terminal");
        }
        let instruction = chip.next_byte();

        match instruction {
            // LDX
            // FLAGS: N Z
            0xA2 => {
                // Mode: Immediate
                // Syntax: LDX #$44
                // Hex: $A2
                // Width: 2
                // Timing: 2
                let arg = chip.next_byte();
                chip.z = arg == 0;
                chip.x = arg;
            }
            // LDA
            // FLAGS: N Z
            0xA9 => {
                // Mode: Immediate
                // Syntax: LDA #$44
                // Hex: $A9
                // Width: 2
                // Timing: 2
                let arg = chip.next_byte();
                chip.z = arg == 0;
                chip.a = arg;
            }
            // STA
            // FLAGS: None
            0x85 => {
                // Mode: Zero Page
                // Syntax: STA $44
                // Hex: $85
                // Width: 2
                // Timing: 3
                let arg = chip.next_byte();
                chip.mmap[arg as u16] = chip.a;
            }
            // STA
            // FLAGS: None
            0x95 => {
                // Mode: Zero Page,X
                // Syntax: STA $44
                // Hex: $95
                // Width: 2
                // Timing: 4
                let arg = chip.next_byte();
                let index = (arg + chip.x) as u16;
                chip.mmap[index] = chip.a;
            }
            // INX
            // FLAGS: N z
            0xE8 => {
                // Syntax: INX
                // Hex: $E8
                // Width: 1
                // Timing: 2
                chip.x = chip.x.wrapping_add(1);
                chip.z = chip.x == 0;
            }
            // Branching Instructions
            //
            // BNE
            // FLAGS:
            0xD0 => {
                // Syntax: BNE Label
                // Hex: $D0
                // Width: 1
                // Timing: 2, +1 if taken, +1 if across page boundry
                let arg = chip.next_byte() as i8;
                if !chip.z {
                    chip.pc = chip.pc.wrapping_add(arg as u16);
                }
            }
            // Stack Instructions
            //
            // TXS
            // FLAGS:
            0x9A => {
                // Syntax: TXS
                // Hex: $9A
                // Width: 1
                // Timing: 2
                chip.sp = chip.x;
            }
            // Subroutine Instructions
            //
            // JSR
            // FLAGS:
            0x20 => {
                // Syntax: JSR
                // Hex: $20
                // Width: 3
                // Timing: 6
                let low = chip.next_byte() as u16;
                let high = chip.next_byte() as u16;
                let jump_address = (high << 8) + low;

                let ret_low = chip.pc as u8;
                let ret_high = (chip.pc >> 8) as u8;
                chip.mmap[chip.sp as u16] = ret_high;
                chip.sp -= 1;
                chip.mmap[chip.sp as u16] = ret_low;
                chip.sp -= 1;
                chip.pc = jump_address
            }
            instruction => {
                if debug {
                    std::thread::sleep(std::time::Duration::from_millis(5000));
                    teardown_terminal().expect("terminal could not be torn down");
                }
                panic!("Unkown instruction: {:02X}", instruction);
            }
        }
        if debug {
            std::thread::sleep(std::time::Duration::from_millis(10));
            draw_terminal(&chip).expect("couldn't draw terminal");
        }
    }
}
