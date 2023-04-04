pub mod components;

use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

use crate::msx::components::{bus::Bus, cpu::Z80, memory::Memory, sound::AY38910, vdp::TMS9918};

use self::components::instruction::Instruction;

// "address": format!("{:04X}", pc),
// "instruction": instr.name(),
// "hexcontents": instr.opcode_with_args(),

#[derive(Clone, PartialEq)]
pub struct ProgramEntry {
    pub address: String,
    pub instruction: String,
    pub data: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Msx {
    pub cpu: Z80,
    pub vdp: TMS9918,
    pub psg: AY38910,

    current_scanline: u16,

    // debug options
    pub breakpoints: Vec<u16>,
    pub max_cycles: Option<u64>,
    pub open_msx: bool,
    pub break_on_mismatch: bool,
    pub track_flags: bool,
    pub previous_memory: Option<Vec<u8>>,
    pub memory_hash: u64,
}

impl Msx {
    pub fn new() -> Self {
        println!("Initializing MSX...");
        let bus = Arc::new(RwLock::new(Bus::new()));
        let memory = Memory::new(bus.clone(), 64 * 1024);
        let cpu = Z80::new(bus, memory);

        Self {
            cpu,
            current_scanline: 0,
            max_cycles: None,
            track_flags: false,
            vdp: TMS9918::new(),
            psg: AY38910::new(),
            open_msx: false,
            break_on_mismatch: false,
            breakpoints: Vec::new(),
            previous_memory: None,
            memory_hash: 0,
        }
    }

    pub fn add_breakpoint(&mut self, address: u16) {
        self.breakpoints.push(address);
    }

    #[allow(unused)]
    pub fn load_binary(&mut self, path: &str, load_address: u16) -> std::io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        for (i, byte) in buffer.iter().enumerate() {
            let address = load_address.wrapping_add(i as u16);
            self.cpu.memory.write_byte(address, *byte);
        }

        Ok(())
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> anyhow::Result<()> {
        self.cpu.memory.load_bios(rom)?;

        Ok(())
    }

    pub fn load_bios(&mut self, path: PathBuf) -> std::io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        self.cpu.memory.load_bios(&buffer)?;

        Ok(())
    }

    pub fn program(&self) -> Vec<ProgramEntry> {
        let mut program = Vec::new();
        let mut pc = self.cpu.pc;

        loop {
            if pc.checked_add(1).is_none() {
                break;
            }

            if program.len() > 100 {
                break;
            }

            let instr = Instruction::parse(&self.cpu.memory, pc);
            program.push(ProgramEntry {
                address: format!("{:04X}", pc),
                instruction: instr.name().to_string(),
                data: instr.opcode_with_args(),
            });
            pc += instr.len() as u16;
        }

        program
    }

    #[allow(unused)]
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.vdp.reset();
        self.psg.reset();
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.cpu.max_cycles = self.max_cycles;
        self.cpu.track_flags = self.track_flags;

        let mut stop_next = false;

        loop {
            self.cpu.execute_cycle();

            let mut stop = false;

            if self.breakpoints.contains(&self.cpu.pc) {
                println!("Breakpoint hit at {:#06X}", self.cpu.pc);
                stop = true;
            }

            if stop || stop_next {
                if stop_next {
                    println!("Stepped to {:#06X}", self.cpu.pc);
                }
                stop_next = false;
            }

            if self.cpu.halted {
                break;
            }

            self.current_scanline = (self.current_scanline + 1) % 192;
            if self.current_scanline == 0 {
                // renderer.draw(0, 0, 256, 192);
                // display.update_screen(&renderer.screen_buffer);
            }
        }

        Ok(())
    }
}
