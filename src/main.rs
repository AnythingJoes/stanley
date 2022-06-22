use std::{
    error::Error,
    fs,
    time::{Duration, Instant},
};

use clap::Parser;

mod system;
use system::instructions::Instruction;
use system::System;

mod timer;
use timer::Timer;

mod debugger;
use debugger::{get_debugger, try_parse_breakpoint, BreakPointType};

mod renderer;
use renderer::{Renderer, WindowEvent};

mod recorder;
use recorder::Recorder;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    debug: bool,
    #[clap(long)]
    disassemble: bool,
    /// Record your session, taking a screenshot when you exit. The screenshot and recording of
    /// your session are placed in the tests/snapshots/<SNAPSHOT_NAME>/ directory. These will be
    /// picked up by the automated test system.
    #[clap(long, value_name = "SNAPSHOT_NAME")]
    record: Option<String>,
    // TODO: take hex argument
    #[clap(short, long, parse(try_from_str=try_parse_breakpoint))]
    breakpoint: Option<BreakPointType>,
    #[clap(short, long)]
    symbol_file: Option<String>,
    file_name: String,
}

fn main() -> Result<()> {
    let Args {
        debug,
        disassemble,
        record,
        breakpoint,
        symbol_file,
        file_name,
    } = Args::parse();

    let byte_vec = fs::read(&file_name).map_err(|e| e.to_string())?;
    let program = byte_vec
        .try_into()
        .expect("Program expected to be 4096 bytes was not");
    let mut debugger = get_debugger(debug);
    let mut recorder_option = record
        .map(|snapshot_name| Recorder::new(&snapshot_name, &file_name))
        .transpose()?;

    if debug && disassemble {
        debugger.dump_disassembly(program);
        return Ok(());
    }

    debugger.setup(program, breakpoint, symbol_file)?;

    let mut system = System::new(program);
    let total_time = Instant::now();
    let mut renderer = Renderer::setup()?;

    // Timing stuff
    let mut previous_clocks = 0;
    let mut timer = Timer::start();

    loop {
        let clocks_run = system.clocks - previous_clocks;
        if timer.should_render() && (debug || !system.tia.is_drawing()) {
            renderer.render(&system.tia.buffer)?;
            timer.did_render();
        }

        if clocks_run > 10 {
            let clock_time = Duration::from_nanos((clocks_run * 837) as u64);
            timer.pause_for(clock_time);
            previous_clocks = system.clocks;
        }

        if let Err(e) = debugger.debug_loop(&system) {
            eprintln!("{}", e);
            break;
        }

        let event = renderer.handle_events();
        if let Some(recorder) = recorder_option.as_mut() {
            recorder.update(&event, &system)?;
        }

        match event {
            WindowEvent::Quit => break,
            WindowEvent::None => (),
            event => system.input_event(&event),
        };

        let instruction: Instruction = system.next_byte().try_into()?;

        if let Err(e) = system.execute(instruction) {
            eprintln!("Time: {}", total_time.elapsed().as_nanos());
            eprintln!("Clocks: {}", system.clocks);
            eprintln!("{}", e);
            break;
        }
    }
    debugger.teardown()?;
    Ok(())
}
