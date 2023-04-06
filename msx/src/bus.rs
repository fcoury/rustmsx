use serde::{Deserialize, Serialize};
use tracing::error;

use super::{ppi::Ppi, sound::AY38910, vdp::TMS9918};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Bus {
    slot_count: u8,

    // I/O Devices
    pub vdp: TMS9918,
    pub psg: AY38910,
    pub ppi: Ppi,

    vdp_io_clock: u8,
    primary_slot_config: u8,
    slot3_secondary_config: u8,
}

impl Default for Bus {
    fn default() -> Self {
        let slot_count = 4;

        Self {
            slot_count,
            vdp: TMS9918::new(),
            psg: AY38910::new(),
            ppi: Ppi::new(),
            vdp_io_clock: 0,
            primary_slot_config: 0,
            slot3_secondary_config: 0,
        }
    }
}

impl Bus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.vdp.reset();
        self.psg.reset();
        self.ppi.reset();
    }

    pub fn input(&mut self, port: u8) -> u8 {
        match port {
            0x98 | 0x99 => self.vdp.read(port),
            0xA0 | 0xA1 => self.psg.read(port),
            0xA8 | 0xA9 | 0xAA | 0xAB => self.ppi.read(port),
            _ => {
                error!("  *** [BUS] Invalid port {:02X} read", port);
                0xff
            }
        }
    }

    pub fn output(&mut self, port: u8, data: u8) {
        match port {
            0x98 | 0x99 => self.vdp.write(port, data),
            0xA0 | 0xA1 => self.psg.write(port, data),
            0xA8 | 0xA9 | 0xAA | 0xAB => self.ppi.write(port, data),
            _ => {
                error!("  *** [BUS] Invalid port {:02X} write", port);
            }
        };
    }
}
