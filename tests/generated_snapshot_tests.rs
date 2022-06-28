use std::fs;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::string::String;

use stanley::renderer::WindowEvent;
use stanley::system::tia::{BUFF_SIZE, HEIGHT, STRIDE, WIDTH};
use stanley::system::{instructions::Instruction, System};
use stanley::Result;

const OUTPUT_WIDTH: usize = WIDTH as usize * OUTPUT_PIXEL_WIDTH;
const OUTPUT_HEIGHT: usize = HEIGHT as usize * OUTPUT_PIXEL_HEIGHT;
const OUTPUT_STRIDE: usize = 3;
const OUTPUT_PIXEL_HEIGHT: usize = 4;
const OUTPUT_PIXEL_WIDTH: usize = 6;

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

    let any_differences = screen != screen_actual;
    if any_differences {
        let buffer = screen
            .chunks(STRIDE)
            .zip(screen_actual.chunks(STRIDE))
            .map(|(expected, actual)| {
                (
                    [expected[2], expected[1], expected[0]],
                    [actual[2], actual[1], actual[0]],
                )
            })
            .map(|(expected, actual)| {
                if expected == actual {
                    let row = [expected; OUTPUT_PIXEL_WIDTH];
                    [row; OUTPUT_PIXEL_HEIGHT]
                } else {
                    let mut row = [[0; OUTPUT_STRIDE]; OUTPUT_PIXEL_WIDTH];
                    let mut output_pixel = [row; OUTPUT_PIXEL_HEIGHT];
                    let red = [0xFF, 0x00, 0x00];

                    row[0] = red;
                    let diff_pixel_width = (OUTPUT_PIXEL_WIDTH - 2) / 2;
                    for item in row.iter_mut().skip(1).take(diff_pixel_width) {
                        *item = expected;
                    }
                    for item in row
                        .iter_mut()
                        .skip(1 + diff_pixel_width)
                        .take(diff_pixel_width)
                    {
                        *item = actual;
                    }
                    row[row.len() - 1] = red;
                    output_pixel[0] = [red; OUTPUT_PIXEL_WIDTH];
                    for item in output_pixel
                        .iter_mut()
                        .skip(1)
                        .take(OUTPUT_PIXEL_HEIGHT - 2)
                    {
                        *item = row;
                    }
                    output_pixel[output_pixel.len() - 1] = [red; OUTPUT_PIXEL_WIDTH];
                    output_pixel
                }
            })
            .collect::<Vec<_>>()
            .chunks(WIDTH as usize)
            .flat_map(|row| {
                let mut output: Vec<Vec<[u8; OUTPUT_STRIDE]>> = vec![vec![]; OUTPUT_PIXEL_HEIGHT];
                for i in 0..output.len() {
                    output.push(row.iter().flat_map(|pixel| pixel[i]).collect());
                }
                output.into_iter().flatten()
            })
            .flatten()
            .collect::<Vec<u8>>();
        create_ppm(&buffer, test_name).unwrap();
    }

    assert!(
        !any_differences,
        "Unexpected image output, view diff at ./artifacts/{}.ppm",
        test_name
    )
}

fn create_ppm(buffer: &[u8], output_name: &str) -> Result<()> {
    fs::create_dir_all("./artifacts")?;

    let mut ppm_image = fs::File::create(format!("./artifacts/{output_name}.ppm"))?;
    writeln!(ppm_image, "P6")?;
    writeln!(ppm_image, "{} {}", OUTPUT_WIDTH, OUTPUT_HEIGHT)?;
    writeln!(ppm_image, "255")?;

    ppm_image.write_all(buffer)?;
    Ok(())
}
