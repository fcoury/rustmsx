#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use tracing::{error, info, trace};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub pattern: u32,
    pub color: u8,
    pub collision: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TMS9918 {
    #[serde(with = "BigArray")]
    pub vram: [u8; 0x4000],
    pub data_pre_read: u8, // read-ahead value
    pub registers: [u8; 8],
    pub status: u8,
    pub address: u16,
    pub first_write: Option<u8>,
    #[serde(with = "BigArray")]
    pub screen_buffer: [u8; 256 * 192],
    pub sprites: [Sprite; 8],
    pub frame: u8,
    pub line: u8,
    pub vblank: bool,
}

impl Default for TMS9918 {
    fn default() -> Self {
        Self {
            vram: [0; 0x4000],
            data_pre_read: 0,
            registers: [0; 8],
            status: 0,
            address: 0,
            first_write: None,
            screen_buffer: [0; 256 * 192],
            sprites: [Sprite {
                x: 0,
                y: 0,
                pattern: 0,
                color: 0,
                collision: false,
            }; 8],
            frame: 0,
            line: 0,
            vblank: false,
        }
    }
}

impl TMS9918 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.vram = [0; 0x4000];
        self.data_pre_read = 0;
        self.registers = [0; 8];
        self.status = 0;
        self.address = 0;
        self.first_write = None;
        self.screen_buffer = [0; 256 * 192];
        self.sprites = [Sprite {
            x: 0,
            y: 0,
            pattern: 0,
            color: 0,
            collision: false,
        }; 8];
        self.frame = 0;
        self.line = 0;
        self.vblank = false;
    }

    // Pattern Table Base Address = register 2 * 0x400
    pub fn pattern_table(&self) -> &[u8] {
        let base_address = self.registers[2] as usize * 0x400;
        let pattern_table_size = 256 * 8; // Assuming 256 characters, 8 bytes per character
        &self.vram[base_address..(base_address + pattern_table_size)]
    }

    pub fn vram_read_np(&self, address: usize) -> usize {
        self.vram[address & 0x3FFF] as usize
    }

    pub fn get_vertical_scroll(&self) -> usize {
        // Replace with the correct logic to get the vertical scroll value
        0
    }

    // WebMSX input98
    fn read_vram(&mut self) -> u8 {
        // uses the read-ahead value
        let data = self.data_pre_read;

        // pre-read the next value
        self.data_pre_read = self.vram[self.address as usize];

        // increment the address
        self.address = self.address.wrapping_add(1);

        // reset the latch
        self.first_write = None;

        // return the read-ahead value
        data
    }

    fn write_98(&mut self, data: u8) {
        if data == 0x63 {
            info!(
                "[VDP] Writing at {:04X}: 0x{:02X} ({}) on port #98, handling...",
                self.address, data, data as char
            );
        }
        if self.address < 0x4000 {
            self.vram[self.address as usize] = data;
            self.data_pre_read = data;
        } else {
            // error!(
            //     "[VDP] Attempted to write to VRAM at address {:04X} = 0x{:02X} {}",
            //     self.address, data, data as char
            // );
        }
        self.address = self.address.wrapping_add(1);
        trace!(
            "[VDP] Address after increment: 0x{:04X}, removing latched data...",
            self.address
        );

        self.first_write = None;
        trace!("");
    }

    // fn read_register(&mut self) -> u8 {
    //     let data = self.status;
    //     // TODO: m_StatusReg = m_FifthSprite;
    //     // TODO: check_interrupt();
    //     self.latch = false;
    //     data
    // }

    fn read_register(&mut self) -> u8 {
        self.first_write = None;
        let res = self.status;
        // TODO: disable interrupt
        self.status &= 0x7F;
        res
    }

    fn write_99(&mut self, data: u8) {
        info!(
            "[VDP] Received 0x{:02X} ({}) at Port #99, handling...",
            data, data as char
        );
        if let Some(latched_value) = self.first_write {
            info!(
                "[VDP] Received data after latch: 0x{:02X} (checks if 0x{:02X} == 0)",
                data,
                data & 0x80
            );
            if data & 0x80 == 0 {
                // Set register
                info!("[VDP] Set register: {:02X}", data);
                let reg = data & 0x07;
                info!("[VDP] Register is: {:08b}", reg);
                self.registers[reg as usize] = latched_value;
                info!("[VDP] Current latched value: {:02X}", latched_value);
                // On V9918, the VRAM pointer high gets also written when writing to registers
                self.address = (self.address & 0x00FF) | ((latched_value as u16 & 0x03F) << 8);
                info!(
                    "[VDP] Also setting high part of the address to {:02X}. Address after: {:04X}",
                    latched_value, self.address
                );
            } else {
                // Set VRAM pointer
                info!(
                    "[VDP] Latched value: 0x{:02X}. Received: 0x{:02X}",
                    latched_value, data
                );
                info!("[VDP] Current address: 0x{:04X}", self.address);

                // extracts the 6-bit most significant bits
                let msb = (data & 0x3F) as u16;
                let lsb = latched_value as u16;

                info!("[VDP] MSB: {:08b} LSB: {:08b}", msb, lsb);
                self.address = self.address | msb | lsb;
                info!("[VDP] Address after MSB, MLB: {:04X}", self.address);
                // Pre-read VRAM if "writemode = 0"
                if (data & 0x40) == 0 {
                    self.status = self.vram[self.address as usize];
                    self.address = self.address.wrapping_add(1);
                    info!("[VDP] Writemode is 0, address after: {:04X}", self.address);
                }
            }
            self.first_write = None;
        } else {
            self.first_write = Some(data);
            // On V9918, the VRAM pointer low gets written right away
            // println!("Address before: {:04X}", self.address);
            self.address = (self.address & 0xFF00) | data as u16;
            // println!("Address after: {:04X}", self.address);
            info!(
                "[VDP] Received first byte of the address (0x{:02X}), latching...",
                data
            );
        }
        info!("");
    }

    pub fn read(&mut self, port: u8) -> u8 {
        match port {
            // VRAM Read
            0x98 => self.read_vram(),
            // Register read
            0x99 => self.read_register(),
            _ => {
                error!("Invalid port: {:02X}", port);
                0xFF
            }
        }
    }

    pub fn write(&mut self, port: u8, data: u8) {
        // writing to data port 0x98
        match port {
            0x98 => self.write_98(data),
            0x99 => self.write_99(data),
            _ => {
                error!("Invalid port: {:02X}", port);
            }
        }
    }
}
