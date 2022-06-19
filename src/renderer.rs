use crate::system::tia::{HEIGHT, WIDTH};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    render::{Texture, TextureCreator, WindowCanvas},
    surface::Surface,
    EventPump,
};

pub enum InputType {
    Joystick1Button,
    Joystick1Up,
    Joystick1Down,
    Joystick1Left,
    Joystick1Right,
}

pub enum WindowEvent {
    None,
    Quit,
    InputStart(InputType),
    InputEnd(InputType),
}

pub struct Renderer<'a> {
    event_pump: EventPump,
    canvas: WindowCanvas,
    texture: Texture<'a>,
}

impl<'a> Renderer<'a> {
    pub fn setup() -> super::Result<Renderer<'a>> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window("Stanley: Atari 2600 Emulator", 800, 600)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        let canvas = window.into_canvas().index(find_sdl_gl_driver()?).build()?;
        let texture_creator: &'static TextureCreator<_> =
            Box::leak(Box::new(canvas.texture_creator()));
        // TODO: Should I use a supported opengl pixel formats?
        let surface = Surface::new(WIDTH, HEIGHT, sdl2::pixels::PixelFormatEnum::RGBA8888)?;
        let texture = surface.as_texture(texture_creator)?;

        let event_pump = sdl_context.event_pump()?;
        Ok(Self {
            texture,
            canvas,
            event_pump,
        })
    }

    pub fn render(&mut self, buffer: &crate::system::tia::Buffer) -> super::Result<()> {
        self.texture
            .update(None, &buffer.0, (4 * crate::system::tia::WIDTH) as usize)?;
        self.canvas.copy(&self.texture, None, None)?;
        self.canvas.present();
        Ok(())
    }

    pub fn handle_events(&mut self) -> WindowEvent {
        let mut events = self.event_pump.poll_iter();
        let event = events.next();
        match event {
            // Close events
            Some(
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                },
            ) => WindowEvent::Quit,
            Some(Event::KeyDown {
                keycode: Some(Keycode::F),
                ..
            }) => WindowEvent::InputStart(InputType::Joystick1Button),
            Some(Event::KeyUp {
                keycode: Some(Keycode::F),
                ..
            }) => WindowEvent::InputEnd(InputType::Joystick1Button),
            // Up
            Some(Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            }) => WindowEvent::InputStart(InputType::Joystick1Up),
            Some(Event::KeyUp {
                keycode: Some(Keycode::W),
                ..
            }) => WindowEvent::InputEnd(InputType::Joystick1Up),
            // Down
            Some(Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            }) => WindowEvent::InputStart(InputType::Joystick1Down),
            Some(Event::KeyUp {
                keycode: Some(Keycode::S),
                ..
            }) => WindowEvent::InputEnd(InputType::Joystick1Down),
            // Left
            Some(Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            }) => WindowEvent::InputStart(InputType::Joystick1Left),
            Some(Event::KeyUp {
                keycode: Some(Keycode::A),
                ..
            }) => WindowEvent::InputEnd(InputType::Joystick1Left),
            // Right
            Some(Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            }) => WindowEvent::InputStart(InputType::Joystick1Right),
            Some(Event::KeyUp {
                keycode: Some(Keycode::D),
                ..
            }) => WindowEvent::InputEnd(InputType::Joystick1Right),
            _ => WindowEvent::None,
        }
    }
}

fn find_sdl_gl_driver() -> super::Result<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Ok(index as u32);
        }
    }
    Err("Couldn't find gl driver".into())
}
