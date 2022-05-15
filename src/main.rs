use std::fs;

#[derive(Debug, Default)]
struct Nmos6502 {
    x: u8,
}

fn main() {
    let mut byte_vec = fs::read("./tictactoe.bin").unwrap();
    let mut bytes = byte_vec.iter_mut();
    let mut chip = Nmos6502::default();

    loop {
        let byte = bytes.next().expect("End of program");
        match byte {
            0xA2 => {
                // Mode: Immediate
                // Syntax: LDX #$44
                // Hex: $A2
                // Width: 2
                // Timing: 2
                let arg = bytes.next().expect("Expected arg for 0xA2 got end of file");
                chip.x = *arg;
            }
            instruction => {
                panic!("Unkown instruction: {:X}", instruction);
            }
        }
    }
}
