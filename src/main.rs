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
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    debug: bool,
}

#[derive(Debug)]
struct Nmos6502 {
    // X indexing register
    x: u8,
    // A accumulator register
    a: u8,
    memory: [u8; 256],
    // Program counter
    pc: u16,
    // Stack pointer
    sp: u8,
    // Address range 0x1000 to 0x2000
    program: [u8; 4096],
    // FLAGS
    // zero
    z: bool,
}

impl Nmos6502 {
    fn new(program: [u8; 4096]) -> Self {
        Nmos6502 {
            program,
            x: 0,
            a: 0,
            z: false,
            memory: [0; 256],
            pc: 0x1000,
            sp: 0,
        }
    }

    fn next_byte(&mut self) -> u8 {
        let byte = self.program[(self.pc - 0x1000) as usize];
        self.pc += 1;
        byte
    }
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
        Print(format!(
            "Next Instruction: {:02X}",
            chip.program[(chip.pc - 0x1000) as usize]
        )),
        cursor::MoveToNextLine(2),
        cursor::MoveRight(5),
        Print("MMappd Hardware"),
        cursor::MoveToNextLine(1),
    )?;

    for i in 0..8 {
        for j in 0..16 {
            let memory = chip.memory[i * 16 + j];
            queue!(stdout, Print(format!("{:02X} ", memory)))?
        }

        queue!(stdout, cursor::MoveToNextLine(1))?
    }
    queue!(
        stdout,
        cursor::MoveToNextLine(2),
        Print("RAM"),
        cursor::MoveToNextLine(1),
    )?;
    for i in 0..8 {
        for j in 0..16 {
            let memory = chip.memory[128 + i * 16 + j];
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
                chip.memory[arg as usize] = chip.a;
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
                let index = (arg + chip.x) as usize;
                chip.memory[index] = chip.a;
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
