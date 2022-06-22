use std::fs;
use std::io::Write;
use std::path::Path;

use crate::renderer::WindowEvent;
use crate::system::System;
use crate::Result;

pub struct Recorder {
    recording: fs::File,
}

impl Recorder {
    pub fn new(snapshot_name: String) -> Result<Self> {
        let path = {
            let tmp_path = Path::new("./tests");
            tmp_path.join(snapshot_name)
        };
        fs::create_dir_all(&path)?;
        let recording = fs::File::create(&path.join("recording.txt"))?;
        Ok(Self { recording })
    }

    pub fn update(&mut self, event: &WindowEvent, system: &System) -> Result<()> {
        if let WindowEvent::None = event {
            return Ok(());
        }
        writeln!(self.recording, "{} {event:?}", system.clocks)?;
        Ok(())
    }
}
