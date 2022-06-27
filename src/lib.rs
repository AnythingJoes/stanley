use std::error::Error;

pub mod debugger;
pub mod recorder;
pub mod renderer;
pub mod system;
pub mod timer;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
