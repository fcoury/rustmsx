use std::{fmt::Debug, fs::File, io::Read, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum SlotType {
    Empty(EmptySlot),
    Ram(RamSlot),
    Rom(RomSlot),
}

#[typetag::serde(tag = "type")]
pub trait Slot: Debug {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EmptySlot;

impl EmptySlot {
    pub fn new() -> Self {
        Self::default()
    }
}

#[typetag::serde]
impl Slot for EmptySlot {
    fn read(&self, _address: u16) -> u8 {
        0xFF
    }

    fn write(&mut self, _address: u16, _value: u8) {}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RomSlot {
    pub rom_path: Option<PathBuf>,
    pub base: u16,
    pub size: u16,
    pub data: Vec<u8>,
}

impl RomSlot {
    pub fn new(rom: &[u8], base: u16, size: u16) -> Self {
        let mut data = vec![0xFF; size as usize];
        data[0..rom.len()].copy_from_slice(rom);
        RomSlot {
            base,
            size,
            data,
            rom_path: None,
        }
    }

    pub fn load(rom_path: PathBuf, base: u16, size: u16) -> anyhow::Result<Self> {
        let mut file = File::open(&rom_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut rom_slot = Self::new(&buffer, base, size);
        rom_slot.rom_path = Some(rom_path);

        Ok(rom_slot)
    }

    fn translate_address(&self, address: u16) -> u16 {
        address - self.base
    }
}

#[typetag::serde]
impl Slot for RomSlot {
    fn read(&self, address: u16) -> u8 {
        let address = self.translate_address(address);
        self.data[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        let address = self.translate_address(address);
        self.data[address as usize] = value;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RamSlot {
    pub base: u16,
    pub size: u16,
    pub data: Vec<u8>,
}

impl RamSlot {
    pub fn new(base: u16, size: u16) -> Self {
        let data = vec![0xFF; size as usize];
        RamSlot { base, data, size }
    }

    fn translate_address(&self, address: u16) -> u16 {
        address - self.base
    }
}

#[typetag::serde]
impl Slot for RamSlot {
    fn read(&self, address: u16) -> u8 {
        let address = self.translate_address(address);
        self.data[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        let address = self.translate_address(address);
        self.data[address as usize] = value;
    }
}
