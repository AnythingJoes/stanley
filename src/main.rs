use std::{
    error::Error,
    fs,
    io::{stdout, Write},
    time::{Duration, Instant},
};

use sdl2::{event::Event, keyboard::Keycode, surface::Surface};

use clap::Parser;
use crossterm::style::Color;
use crossterm::{
    cursor,
    event::{poll, read, Event as CTEvent, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
};

mod system;
use system::instructions::InstructionValue;
use system::tia::{HEIGHT, WIDTH};
use system::System;

mod timer;
use timer::Timer;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    debug: bool,
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

fn draw_terminal(system: &mut System) -> Result<()> {
    let mut stdout = stdout();
    queue!(
        stdout,
        style::SetForegroundColor(Color::White),
        Print(format!("{}", system.chip)),
        cursor::MoveToNextLine(1),
        Print(format!("{}", system)),
    )?;
    queue!(stdout, cursor::MoveToNextLine(1),)?;
    let riot = &system.riot;
    queue!(stdout, Print(format!("{} ", riot)))?;
    queue!(
        stdout,
        cursor::MoveToNextLine(1),
        Print(format!("{} ", system.tia)),
    )?;

    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let Args { debug } = Args::parse();

    if debug {
        set_up_terminal().expect("terminal could not be setup");
    }

    let byte_vec = fs::read("./tictactoe.bin").unwrap();
    let program = byte_vec
        .try_into()
        .expect("Program expected to be 4096 bytes was not");
    let mut system = System::new(program);
    let total_time = Instant::now();

    // Timing stuff
    let mut previous_clocks = 0;
    let mut timer = Timer::start();

    // SDL stuff
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Stanley: Atari 2600 Emulator", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .index(find_sdl_gl_driver().ok_or("Couldn't find gl driver")?)
        .build()?;
    let texture_creator = canvas.texture_creator();
    // TODO: Should I use a supported opengl pixel formats?
    let surface = Surface::new(WIDTH, HEIGHT, sdl2::pixels::PixelFormatEnum::RGBA8888)?;
    let mut texture = surface.as_texture(&texture_creator)?;

    let mut event_pump = sdl_context.event_pump()?;

    'atari_loop: loop {
        if debug {
            clear_terminal().expect("couldn't clear terminal");
            if let Ok(true) = poll(Duration::from_millis(10)) {
                if let Ok(CTEvent::Key(KeyEvent { code, modifiers })) = read() {
                    if code == KeyCode::Esc
                        || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL))
                    {
                        break;
                    }
                }
            }
            texture.update(None, &system.tia.buffer.0, (4 * WIDTH) as usize)?;
            canvas.copy(&texture, None, None)?;
            canvas.present();
            timer.did_render();
        } else {
            let clocks_run = system.clocks - previous_clocks;
            if timer.should_render() && !system.tia.is_drawing() {
                texture.update(None, &system.tia.buffer.0, (4 * WIDTH) as usize)?;
                canvas.copy(&texture, None, None)?;
                canvas.present();
                timer.did_render();
            }

            if clocks_run > 10 {
                let clock_time = Duration::from_nanos((clocks_run * 837) as u64);
                timer.pause_for(clock_time);
                previous_clocks = system.clocks;
            }
        }

        let events = event_pump.poll_iter();
        for event in events {
            match event {
                // Close events
                Event::Quit { .. } => break 'atari_loop,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'atari_loop,
                _ => (),
            }
        }

        let instruction: InstructionValue = system.next_byte().try_into()?;

        if let Err(e) = system.execute(instruction) {
            if debug {
                std::thread::sleep(std::time::Duration::from_millis(5000));
                break;
            }
            eprintln!("Time: {}", total_time.elapsed().as_nanos());
            eprintln!("Clocks: {}", system.clocks);
            return Err(e);
        }
        if debug {
            draw_terminal(&mut system).expect("couldn't draw terminal");
        }
    }
    if debug {
        teardown_terminal().expect("terminal could not be torn down");
    }
    Ok(())
}

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}
