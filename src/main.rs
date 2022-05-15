use std::{
    error::Error,
    fs,
    time::{Duration, Instant},
};

use clap::Parser;

mod system;
use system::instructions::InstructionValue;
use system::System;

mod timer;
use timer::Timer;

mod debugger;
use debugger::Debugger;

mod renderer;
use renderer::Renderer;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let Args { debug } = Args::parse();

    let debugger = Debugger::mode(debug);
    debugger.setup()?;

    let mut renderer = Renderer::setup()?;

    let byte_vec = fs::read("./tictactoe.bin").unwrap();
    let program = byte_vec
        .try_into()
        .expect("Program expected to be 4096 bytes was not");
    let mut system = System::new(program);
    let total_time = Instant::now();

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

        let instruction: InstructionValue = system.next_byte().try_into()?;

        if let Err(e) = system.execute(instruction) {
            eprintln!("Time: {}", total_time.elapsed().as_nanos());
            eprintln!("Clocks: {}", system.clocks);
            eprintln!("{}", e);
            break;
        }
        if let Err(e) = debugger.debug_loop(&system) {
            eprintln!("{}", e);
            break;
        }
        if let Err(e) = renderer.handle_events() {
            eprintln!("{}", e);
            break;
        }
    }
    debugger.teardown()?;
    Ok(())
}
