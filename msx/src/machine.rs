use std::{
    fmt,
    fs::File,
    io::Read,
    sync::{Arc, RwLock},
};

use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::{bus::Bus, cpu::Z80, instruction::Instruction, memory::Memory, vdp::TMS9918};

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramEntry {
    pub address: u16,
    pub instruction: String,
    pub data: String,
}

impl fmt::Display for ProgramEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04X}  {:<10}  {}",
            self.address, self.data, self.instruction
        )
    }
}

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Clone, Debug, PartialEq, Eq)]
pub struct Msx {
    #[serde(skip)]
    #[derivative(PartialEq = "ignore")]
    pub bus: Arc<RwLock<Bus>>,
    pub cpu: Z80,

    pub current_scanline: u16,
    running: bool,

    // debug options
    pub breakpoints: Vec<u16>,
    pub max_cycles: Option<u64>,
    pub open_msx: bool,
    pub break_on_mismatch: bool,
    pub track_flags: bool,
    pub previous_memory: Option<Vec<u8>>,
    pub memory_hash: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    RomLoaded,
    Breakpoint,
    Halt,
    MemoryUpdate { address: u16, value: u8 },
    Cycle,
}

impl Default for Msx {
    fn default() -> Self {
        println!("Initializing MSX...");
        let bus = Arc::new(RwLock::new(Bus::new()));
        let memory = Memory::new(bus.clone(), 64 * 1024);
        let cpu = Z80::new(bus.clone(), memory);

        Self {
            cpu,
            bus,
            current_scanline: 0,
            max_cycles: None,
            track_flags: false,
            open_msx: false,
            break_on_mismatch: false,
            breakpoints: Vec::new(),
            previous_memory: None,
            memory_hash: 0,
            running: false,
        }
    }
}

impl Msx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_vdp(&self) -> TMS9918 {
        let bus = self.bus.read().unwrap();
        bus.vdp.clone()
    }

    pub fn ram(&self) -> Vec<u8> {
        self.cpu.memory.data.to_vec()
    }

    pub fn vram(&self) -> Vec<u8> {
        let bus = self.bus.read().unwrap();
        bus.vdp.vram.to_vec()
    }

    #[allow(unused)]
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

    pub fn load_rom(&mut self, rom: &[u8]) -> std::io::Result<()> {
        self.reset();
        self.cpu.memory.load_bios(rom)?;

        Ok(())
    }

    pub fn instruction(&self) -> ProgramEntry {
        let instr = Instruction::parse(&self.cpu.memory, self.cpu.pc);
        ProgramEntry {
            address: self.cpu.pc,
            instruction: instr.name(),
            data: instr.opcode_with_args(),
        }
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
                address: pc,
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
        let mut bus = self.bus.write().unwrap();
        bus.reset();
    }

    // pub fn run(&mut self) -> anyhow::Result<()> {
    //     self.running = true;
    //     self.cpu.max_cycles = self.max_cycles;
    //     self.cpu.track_flags = self.track_flags;

    //     let mut stop_next = false;

    //     loop {
    //         self.cpu.execute_cycle();
    //         self.notify(EventType::Cycle);

    //         let mut stop = !self.running;

    //         if self.breakpoints.contains(&self.cpu.pc) {
    //             println!("Breakpoint hit at {:#06X}", self.cpu.pc);
    //             stop = true;
    //         }

    //         if stop || stop_next {
    //             if stop_next {
    //                 println!("Stepped to {:#06X}", self.cpu.pc);
    //             }
    //             stop_next = false;
    //         }

    //         if self.cpu.halted {
    //             break;
    //         }

    //         self.current_scanline = (self.current_scanline + 1) % 192;
    //         if self.current_scanline == 0 {
    //             // renderer.draw(0, 0, 256, 192);
    //             // display.update_screen(&renderer.screen_buffer);
    //         }
    //     }

    //     Ok(())
    // }

    pub fn step(&mut self) {
        self.cpu.execute_cycle();
        self.current_scanline = (self.current_scanline + 1) % 192;
    }

    pub fn halted(&self) -> bool {
        self.cpu.halted
    }
}
