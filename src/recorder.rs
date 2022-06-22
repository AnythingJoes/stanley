use crate::renderer::WindowEvent;
use crate::system::System;

pub struct Recorder {}

impl Recorder {
    pub fn new(snapshot_name: String) -> Self {
        Self {}
    }

    pub fn update(&self, event: &WindowEvent, system: &System) {}
}
