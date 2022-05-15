use std::fs;

#[derive(Debug)]
struct Nmos6502 {
    // X indexing register
    x: u8,
    // A accumulator register
    a: u8,
    memory: [u8; 8192],
}

fn main() {
    let mut byte_vec = fs::read("./tictactoe.bin").unwrap();
    let mut bytes = byte_vec.iter_mut();
    let mut chip = Nmos6502 {
        x: 0,
        a: 0,
        memory: [0; 8192],
    };

    loop {
        let byte = bytes.next().expect("End of program");
        match byte {
            // LDX
            // FLAGS: N Z
            0xA2 => {
                // Mode: Immediate
                // Syntax: LDX #$44
                // Hex: $A2
                // Width: 2
                // Timing: 2
                let arg = bytes.next().expect("Expected arg for 0xA2 got end of file");
                chip.x = *arg;
            }
            // LDA
            // FLAGS: N Z
            0xA9 => {
                // Mode: Immediate
                // Syntax: LDX #$44
                // Hex: $A9
                // Width: 2
                // Timing: 2
                let arg = bytes.next().expect("Expected arg for 0xA9 got end of file");
                chip.a = *arg;
            }
            // STA
            // FLAGS: None
            0x95 => {
                // Mode: Zero Page,X
                // Syntax: STA $44
                // Hex: $95
                // Width: 2
                // Timing: 4
                let arg = bytes.next().expect("Expected arg for 0x95 got end of file");
                let index = (*arg + chip.x) as usize;
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
            }
            instruction => {
                panic!("Unkown instruction: {:X}", instruction);
            }
        }
    }
}
