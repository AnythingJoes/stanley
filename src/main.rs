use std::{
    error::Error,
    fs,
    io::{stdout, Write},
};

pub use crossterm::style::Color;
use crossterm::{
    cursor, execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct Nmos6502 {
    // X indexing register
    x: u8,
    // A accumulator register
    a: u8,
    memory: [u8; 256],
    pc: u16,
    // Address range 0x1000 to 0x2000
    program: [u8; 4096],
    // FLAGS
    // zero
    z: bool,
}

impl Nmos6502 {
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
            "Registers: X({:X}) Y({:X}) A({:X})   |",
            chip.x, 0, chip.pc
        )),
        Print(format!("PC: {:X}   |", chip.pc)),
        cursor::MoveToNextLine(1),
        Print(format!("Flags: Z({})", chip.z)),
        cursor::MoveToNextLine(2),
        cursor::MoveRight(5),
        Print("MMappd Hardware"),
        cursor::MoveToNextLine(1),
    )?;

    for i in 0..8 {
        for j in 0..16 {
            let memory = chip.memory[i * 16 + j];
            queue!(stdout, Print(format!("{:X} ", memory)))?
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
            queue!(stdout, Print(format!("{:X} ", memory)))?
        }

        queue!(stdout, cursor::MoveToNextLine(1))?
    }
    stdout.flush()?;
    Ok(())
}

fn main() {
    set_up_terminal().expect("terminal could not be setup");

    let byte_vec = fs::read("./tictactoe.bin").unwrap();
    let mut chip = Nmos6502 {
        x: 0,
        a: 0,
        z: false,
        memory: [1; 256],
        pc: 0x1000,
        program: byte_vec
            .try_into()
            .expect("Program expected to be 4096 bytes was not"),
    };

    loop {
        clear_terminal().expect("couldn't clear terminal");
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
                // Syntax: LDX #$44
                // Hex: $A9
                // Width: 2
                // Timing: 2
                let arg = chip.next_byte();
                chip.z = arg == 0;
                chip.a = arg;
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
                chip.x += 1;
                chip.z = chip.x == 0;
            }
            // BNE
            // FLAGS:
            0xD0 => {
                // Syntax: BNE Label
                // Hex: $D0
                // Width: 1
                // Timing: 2, +1 if taken, +1 if across page boundry
                let arg = chip.next_byte() as i8;
                if chip.z {
                    chip.pc = chip.pc.wrapping_add(arg as u16);
                }
            }
            instruction => {
                eprintln!("Unkown instruction: {:X}", instruction);
                break;
            }
        }
        draw_terminal(&chip).expect("couldn't draw terminal");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    teardown_terminal().expect("terminal could not be setup");
}
