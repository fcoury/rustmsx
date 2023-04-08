use derivative::Derivative;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::slot::SlotType;

use super::{ppi::Ppi, sound::AY38910, vdp::TMS9918};

#[derive(Derivative, Clone, Serialize, Deserialize)]
#[derivative(Debug, PartialEq)]
pub struct Bus {
    slot_count: u8,

    // I/O Devices
    pub vdp: TMS9918,
    pub psg: AY38910,
    pub ppi: Ppi,

    vdp_io_clock: u8,
    primary_slot_config: u8,

    slots: [SlotType; 4],
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
            primary_slot_config: 0x00,
            slots: [
                SlotType::Empty,
                SlotType::Empty,
                SlotType::Empty,
                SlotType::Empty,
            ],
        }
    }
}

impl Bus {
    pub fn new(slots: &[SlotType]) -> Self {
        Self {
            slot_count: 4,
            vdp: TMS9918::new(),
            psg: AY38910::new(),
            ppi: Ppi::new(),
            vdp_io_clock: 0,
            primary_slot_config: 0x00,
            slots: [
                slots.get(0).unwrap().clone(),
                slots.get(1).unwrap().clone(),
                slots.get(2).unwrap().clone(),
                slots.get(3).unwrap().clone(),
            ],
        }
    }

    pub fn mem_size(&self) -> usize {
        0x10000
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
                error!("[BUS] Invalid port {:02X} read", port);
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
                error!("[BUS] Invalid port {:02X} write", port);
            }
        };
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let slot_number = self.get_slot_number_for_address(addr);
        self.slots[slot_number].read(addr)
    }

    pub fn write_byte(&mut self, addr: u16, data: u8) {
        let slot_number = self.get_slot_number_for_address(addr);
        self.slots[slot_number].write(addr, data);
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let low_byte = (value & 0x00FF) as u8;
        let high_byte = ((value & 0xFF00) >> 8) as u8;
        self.write_byte(address, low_byte);
        self.write_byte(address + 1, high_byte);
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let low_byte = self.read_byte(address) as u16;
        let high_byte = self.read_byte(address + 1) as u16;
        (high_byte << 8) | low_byte
    }

    fn get_slot_number_for_address(&self, addr: u16) -> usize {
        let page = (addr >> 14) & 0x03;
        let shift = page * 2;
        ((self.primary_slot_config >> shift) & 0x03) as usize
    }

    pub fn print_memory_page_info(&self) {
        for page in 0..4 {
            let start_address = page * 0x4000;
            let end_address = start_address + 0x3FFF;
            let slot_number = ((self.primary_slot_config >> (page * 2)) & 0x03) as usize;
            let slot_type = self.slots.get(slot_number).unwrap();

            println!(
                "Memory page {} (0x{:04X} - 0x{:04X}): primary slot {} ({})",
                page, start_address, end_address, slot_number, slot_type
            );
        }
    }
}
