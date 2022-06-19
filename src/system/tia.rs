use std::fmt;

use super::colors::COLOR_MAP;
use crate::renderer::{InputType, WindowEvent};

const COLOR_CLOCKS_PER_LINE: usize = 228;
const COLOR_CLOCKS_PER_FRAME: usize = COLOR_CLOCKS_PER_LINE * SCAN_LINES;
const SCAN_LINES: usize = 262;
const COLOR_CLOCKS_PER_SYSTEM_CLOCK: usize = 3;

pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 192;
pub const STRIDE: usize = 4;
const BUFF_SIZE: usize = (WIDTH * HEIGHT * STRIDE as u32) as usize;
const DRAWING_START_ROW: usize = 40;
const DRAWING_START_COLUMN: usize = 68;
const DRAWING_ROWS: usize = 192;
const DRAWING_COLUMNS: usize = 160;

// TIA Register Constants
const INPT4: u16 = 0xC;

pub struct WsyncClocks {
    pub value: usize,
}

pub struct Buffer(pub [u8; BUFF_SIZE]);
impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, ...]", self.0[0], self.0[1])
    }
}

#[derive(Debug)]
enum Nusize {
    OneCopy,
    Quad,
}

#[derive(Debug)]
pub struct Tia {
    vsync: bool,
    vblank: bool,
    pub wsync: bool,

    set_resp0: bool,

    // colors
    colupf: u8,
    colubk: u8,
    colup0: u8,
    colup1: u8,

    //ctrlpf
    pf_reflected: bool,

    //pf registers
    pf0: u8,
    pf1: u8,
    pf2: u8,

    // color clocks this frame
    color_clocks: usize,

    // Input handling
    joystick1_trigger_pressed: bool,

    // Player 1 Sprite
    nusize0: Nusize,
    resp0: usize,
    grp0: u8,

    pub buffer: Buffer,
}

impl Default for Tia {
    fn default() -> Self {
        Tia {
            // actions to take after clocks are updated
            vsync: false,
            vblank: false,
            wsync: false,

            set_resp0: false,

            // colors
            colupf: 0,
            colubk: 0,
            colup0: 0,
            colup1: 0,

            //ctrlpf
            pf_reflected: false,

            //pf registers
            pf0: 0,
            pf1: 0,
            pf2: 0,

            // color clocks this frame
            color_clocks: 0,

            // input handling
            joystick1_trigger_pressed: false,

            // player info
            nusize0: Nusize::OneCopy,
            resp0: 0,
            grp0: 0,

            buffer: Buffer([0xFF; BUFF_SIZE]),
        }
    }
}

impl Tia {
    pub fn set(&mut self, index: u16, value: u8) {
        match index {
            0x00 => self.vsync = (value & 0x02) != 0,
            // TODO: vblank does other thing on D6 and D7 pins, will need to be implemented
            0x01 => self.vblank = (value & 0x02) != 0,
            0x02 => self.wsync = true,
            // TODO: RSYNC: can be ignored in most cases. There is one game that depends on this being
            // handled correctly
            0x03 => (),
            0x04 => self.set_player1_nusize(value),
            0x05 => (), // Ignored for now
            0x06 => self.colup0 = value,
            0x07 => self.colup1 = value,
            0x08 => self.colupf = value,
            0x09 => self.colubk = value,
            // TODO: other parts of ctrlpf
            0x0A => self.pf_reflected = (value & 0x01) == 1,
            0x0B..=0x0C => (), // Ignored for now
            0x0D => self.pf0 = value & 0xF0,
            0x0E => self.pf1 = value,
            0x0F => self.pf2 = value,
            0x10 => self.set_resp0 = true,
            0x11..=0x1A => (), // Ignored for now
            0x1B => self.grp0 = value,
            0x1C..=0x2C => (), // Ignored for now
            0x2D..=0x3F => (), // Unused
            _ => unreachable!("Tia set not implemented for {:04X} index", index),
        }
    }

    pub fn get(&self, index: u16) -> u8 {
        // TODO: Needs a real implementation
        // If it ends in 0xC, it's trying to read player 0 input in this case 0
        // is pressed and 1 in the sign bit is the default state. We want to
        // return the default state until we implement input
        if (index & 0x000F) == INPT4 {
            return if !self.joystick1_trigger_pressed {
                0b1000_0000
            } else {
                0
            };
        }
        // This is not a valid address, but is used to waste time in some programs.
        if (index & 0x000F) == 0x00E {
            return 0;
        }
        unimplemented!("Tia get not implemented for {:04X} index", index);
    }

    // TODO: Use pf_colors
    pub fn tick(&mut self, clocks: usize) {
        let new_color_clocks = self.color_clocks + clocks * COLOR_CLOCKS_PER_SYSTEM_CLOCK;
        let pf = self.get_playfield();

        for i in self.color_clocks..=new_color_clocks {
            let column = Tia::column(i);
            let line = Tia::row(i);
            if column < DRAWING_COLUMNS && line < DRAWING_ROWS {
                // Playfield Draw
                let pixel = line * WIDTH as usize + column;
                let pixel_start = pixel * STRIDE;
                let pf_index = 40 - column / STRIDE;
                let color = if pf & (1 << (pf_index - 1)) != 0 {
                    COLOR_MAP[self.colupf as usize].as_slice()
                } else {
                    COLOR_MAP[self.colubk as usize].as_slice()
                };
                self.buffer.0[pixel_start..=pixel_start + 3].copy_from_slice(&color);

                // GRP0 Draw
                // grp0[clocks on row - start]
                if let Nusize::Quad = self.nusize0 {
                    let sprite_start = self.resp0.wrapping_sub(DRAWING_START_COLUMN);
                    if column >= sprite_start && column < sprite_start + (8 * 4) {
                        let offset = column - sprite_start;
                        if self.grp0 & 1 << (7 - (offset / 4)) != 0 {
                            self.buffer.0[pixel_start..=pixel_start + 3]
                                .copy_from_slice(&COLOR_MAP[self.colup0 as usize].as_slice());
                        }
                    }
                }
            }
        }
        self.color_clocks =
            (self.color_clocks + clocks * COLOR_CLOCKS_PER_SYSTEM_CLOCK) % COLOR_CLOCKS_PER_FRAME;
    }

    fn wsync_ticks(&self) -> usize {
        (COLOR_CLOCKS_PER_LINE - self.color_clocks % COLOR_CLOCKS_PER_LINE)
            / COLOR_CLOCKS_PER_SYSTEM_CLOCK
    }

    pub fn is_drawing(&self) -> bool {
        Tia::row(self.color_clocks) < DRAWING_ROWS
    }

    /// Sync syncs the tia, and returns a number of ticks to advance the clock. Used for the wsync
    /// signal
    pub fn sync(&mut self) -> WsyncClocks {
        if self.set_resp0 {
            // TODO; figure out why I have to add 6 clock cycles to the position to get it in the
            // right place. I can't find anything easily online about a delay
            self.resp0 = self.beam_position() + 6;
            self.set_resp0 = false;
        }

        if self.wsync {
            let clocks = WsyncClocks {
                value: self.wsync_ticks(),
            };
            if self.vsync {
                self.color_clocks = 228 * 3;
            }
            self.wsync = false;
            return clocks;
        }
        WsyncClocks { value: 0 }
    }

    /// Handles an input start or end event from the window, updating its internal state to match.
    pub fn input_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::InputStart(InputType::Joystick1Button) => {
                self.joystick1_trigger_pressed = true
            }
            WindowEvent::InputEnd(InputType::Joystick1Button) => {
                self.joystick1_trigger_pressed = false
            }
            _ => (),
        }
    }

    fn get_playfield(&self) -> u64 {
        let playfield = ((self.pf0.reverse_bits() as u64) << 16)
            + ((self.pf1 as u64) << 8)
            + (self.pf2.reverse_bits() as u64);
        (playfield << 20)
            + if self.pf_reflected {
                playfield.reverse_bits() >> (64 - 20)
            } else {
                playfield
            }
    }

    fn column(color_clocks: usize) -> usize {
        (color_clocks % COLOR_CLOCKS_PER_LINE).wrapping_sub(DRAWING_START_COLUMN)
    }

    fn row(color_clocks: usize) -> usize {
        (color_clocks / COLOR_CLOCKS_PER_LINE).wrapping_sub(DRAWING_START_ROW)
    }

    fn scan_line(&self) -> usize {
        self.color_clocks / COLOR_CLOCKS_PER_LINE
    }

    fn beam_position(&self) -> usize {
        self.color_clocks % COLOR_CLOCKS_PER_LINE
    }

    fn set_player1_nusize(&mut self, value: u8) {
        self.nusize0 = match value {
            0x00 => Nusize::OneCopy,
            0x07 => Nusize::Quad,
            _ => unimplemented!("No size, or number matching {value:02X}"),
        };
    }
}

impl fmt::Display for Tia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "
TIA\r\n
Colors: COLUBK: {:02X} | COLUPF: {:02X} | Current Line: {} | Beam Position: {}\r\n
VSYNC: {} | VBLANK: {}\r\n
Playfields: PF0({:08b}) PF1({:08b}) PF2({:08b})\r\n
Player 1 Sprite: Musize({:?}) RESP0({}) GRP0({:08b})\r\n
            ",
            self.colubk,
            self.colupf,
            self.scan_line(),
            self.beam_position(),
            self.vsync,
            self.vblank,
            self.pf0,
            self.pf1,
            self.pf2,
            self.nusize0,
            self.resp0,
            self.grp0,
        )
    }
}
