use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::renderer::WindowEvent;
use crate::system::System;
use crate::Result;

pub struct Recorder {
    recording: fs::File,
    path: PathBuf,
}

impl Recorder {
    pub fn new(snapshot_name: &str, binary_file_path: &str) -> Result<Self> {
        let path = {
            let tmp_path = Path::new("./tests");
            tmp_path.join(snapshot_name)
        };
        fs::create_dir_all(&path)?;
        let recording = fs::File::create(&path.join("recording.txt"))?;
        let output_binary = path.join("binary.bin");
        fs::copy(binary_file_path, output_binary)?;
        Ok(Self { recording, path })
    }

    pub fn update(&mut self, event: &WindowEvent, system: &System) -> Result<()> {
        match event {
            WindowEvent::Quit => {
                writeln!(self.recording, "{} {event:?}", system.clocks)?;
                fs::write(self.path.join("screen.bin"), system.tia.buffer.0)?;
            }
            WindowEvent::None => {}
            _ => {
                writeln!(self.recording, "{} {event:?}", system.clocks)?;
            }
        }
        Ok(())
    }
}
