use std::fs;

#[derive(Debug)]
struct Nmos6502 {
    // X indexing register
    x: u8,
    // A accumulator register
    a: u8,
    memory: [u8; 8192],
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

fn main() {
    let byte_vec = fs::read("./tictactoe.bin").unwrap();
    let mut chip = Nmos6502 {
        x: 0,
        a: 0,
        z: false,
        memory: [0; 8192],
        pc: 0x1000,
        program: byte_vec
            .try_into()
            .expect("Program expected to be 4096 bytes was not"),
    };

    loop {
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
                panic!("Unkown instruction: {:X}", instruction);
            }
        }
    }
}
