use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::string::String;

use stanley::renderer::WindowEvent;
use stanley::system::tia::{BUFF_SIZE, HEIGHT, STRIDE, WIDTH};
use stanley::system::{instructions::Instruction, System};
use stanley::Result;

include!(concat!(env!("OUT_DIR"), "/tests.rs"));

#[derive(Debug)]
struct AtariInput {
    clock_cycle: usize,
    input: WindowEvent,
}

impl FromStr for AtariInput {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let split_str: Vec<&str> = s.split(' ').collect();
        let clock_cycle: usize = split_str
            .get(0)
            .ok_or_else(|| "Invalid recording file".to_owned())?
            .parse()
            .map_err(|_| "Invalid number in recording file".to_owned())?;
        let input = split_str
            .get(1)
            .ok_or_else(|| "Invalid recording file".to_owned())?
            .parse()?;
        Ok(Self { clock_cycle, input })
    }
}

fn test_snapshot(snapshot_path: impl AsRef<Path>) {
    let recording = fs::read_to_string(snapshot_path.as_ref().join("recording.txt")).unwrap();
    let inputs = recording
        .lines()
        .map(|line| line.parse::<AtariInput>().unwrap());

    let screen: [u8; BUFF_SIZE] = fs::read(snapshot_path.as_ref().join("screen.bin"))
        .unwrap()
        .try_into()
        .unwrap();
    let mut screen_actual: [u8; BUFF_SIZE] = [0; BUFF_SIZE];

    let binary = fs::read(snapshot_path.as_ref().join("binary.bin"))
        .unwrap()
        .try_into()
        .unwrap();

    let mut system = System::new(binary);

    for next_action in inputs {
        loop {
            if next_action.clock_cycle <= system.clocks {
                match next_action.input {
                    WindowEvent::Quit => {
                        screen_actual = system.tia.buffer.0;
                    }
                    e => {
                        system.input_event(&e);
                    }
                }
                break;
            }

            let instruction: Instruction = system.next_byte().try_into().unwrap();
            system.execute(instruction).unwrap();
        }
    }

    let mut differences = screen
        .chunks(STRIDE)
        .zip(screen_actual.chunks(STRIDE))
        .enumerate()
        .filter_map(|(i, (expected, actual))| {
            if expected == actual {
                None
            } else {
                Some((i, expected, actual))
            }
        })
        .peekable();
    let any_differences = differences.peek().is_some();

    if any_differences {}
    assert!(!any_differences)
}
