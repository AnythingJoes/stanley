use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::string::String;

use stanley::renderer::WindowEvent;
use stanley::system::tia::{BUFF_SIZE, HEIGHT, STRIDE, WIDTH};
use stanley::system::{instructions::Instruction, System};
use stanley::Result;

const OUTPUT_WIDTH: usize = (WIDTH * 6) as usize;
const OUTPUT_HEIGHT: usize = (HEIGHT * 4) as usize;
const OUTPUT_STRIDE: usize = 3;

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
    let test_name = snapshot_path
        .as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
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
                Some((i, actual))
            }
        })
        .peekable();
    let any_differences = differences.peek().is_some();

    if any_differences {
        let mut buffer: Vec<Vec<[u8; 3]>> =
            vec![vec![[0; OUTPUT_STRIDE]; OUTPUT_WIDTH]; OUTPUT_HEIGHT];

        let (mut i, mut actual) = differences.next().unwrap();
        let border_pixel = [0xFF, 0x00, 0x00];

        for row in 0..HEIGHT as usize {
            for column in 0..WIDTH as usize {
                let location = row * WIDTH as usize + column;
                let screen_pixel = &screen[location * STRIDE..location * STRIDE + STRIDE];
                let expected_pixel = [screen_pixel[2], screen_pixel[1], screen_pixel[0]];
                let output_row_start = row * 4;
                let output_column_start = column * 6;

                if i == location {
                    let actual_out_pixel = [actual[2], actual[1], actual[0]];
                    for output_row in output_row_start..output_row_start + 4 {
                        for output_column in output_column_start..output_column_start + 6 {
                            let out_pixel = if output_row == output_row_start
                                || output_row == output_row_start + 3
                                || output_column == output_column_start
                                || output_column == output_column_start + 5
                            {
                                border_pixel
                            } else if output_column - output_column_start < 3 {
                                expected_pixel
                            } else {
                                actual_out_pixel
                            };
                            buffer[output_row][output_column] = out_pixel;
                        }
                    }
                    if let Some((idx, a)) = differences.next() {
                        i = idx;
                        actual = a;
                    }
                } else {
                    for output_row in output_row_start..output_row_start + 4 {
                        for output_column in output_column_start..output_column_start + 6 {
                            buffer[output_row][output_column] = expected_pixel;
                        }
                    }
                }
            }
        }
        create_ppm(buffer, test_name).unwrap();
    }

    assert!(
        !any_differences,
        "Unexpected image output, view diff at ./artifacts/{}.ppm",
        test_name
    )
}

fn create_ppm(buffer: Vec<Vec<[u8; 3]>>, output_name: &str) -> Result<()> {
    fs::create_dir_all("./artifacts")?;

    let mut ppm_image = fs::File::create(format!("./artifacts/{output_name}.ppm"))?;
    writeln!(ppm_image, "P6")?;
    writeln!(ppm_image, "{} {}", OUTPUT_WIDTH, OUTPUT_HEIGHT)?;
    writeln!(ppm_image, "255")?;

    for line in buffer {
        for pixel in line {
            ppm_image.write_all(&pixel)?;
        }
    }
    Ok(())
}
