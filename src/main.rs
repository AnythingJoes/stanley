use std::fs;

fn main() {
    let mut byte_vec = fs::read("./tictactoe.bin").unwrap();
    let mut bytes = byte_vec.iter_mut();

    loop {
        let byte = bytes.next().expect("End of program");
        match byte {
            0xA2 => {
                println!("Got A2!")
            }
            instruction => {
                panic!("Unkown instruction: {:X}", instruction);
            }
        }
    }
}
