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

    pub fn pc(&self) -> u16 {
        self.cpu.pc
    }

    pub fn halted(&self) -> bool {
        self.cpu.halted
    }

    pub fn set_a(&mut self, value: u8) {
        self.cpu.a = value;
    }

    pub fn set_b(&mut self, value: u8) {
        self.cpu.b = value;
    }

    pub fn set_c(&mut self, value: u8) {
        self.cpu.c = value;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.cpu.set_hl(value);
    }

    pub fn set_hl_address(&mut self, value: u16) {
        self.cpu.memory.write_word(self.cpu.get_hl(), value);
    }

    pub fn set_memory(&mut self, address: u16, value: u8) {
        self.cpu.memory.write_byte(address, value);
    }

    pub fn get_memory(&self, address: u16) -> u8 {
        self.cpu.memory.read_byte(address)
    }

    #[allow(unused)]
    pub fn add_breakpoint(&mut self, address: u16) {
        self.breakpoints.push(address);
    }

    pub fn load_binary(&mut self, path: &str) -> std::io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        self.cpu.memory.load_bios(&buffer)?;

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

    pub fn program_slice(&self, before_pc: u16, size: u16) -> Vec<ProgramEntry> {
        let mut program = Vec::new();

        let pc = self.cpu.pc;
        let program_start = pc.saturating_sub(before_pc);
        let program_end = program_start + size;

        for pc in program_start..program_end {
            let instr = Instruction::parse(&self.cpu.memory, pc);
            program.push(ProgramEntry {
                address: pc,
                instruction: instr.name().to_string(),
                data: instr.opcode_with_args(),
            });
        }

        program
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

    pub fn vdp(&self) -> TMS9918 {
        let bus = self.bus.read().unwrap();
        bus.vdp.clone()
    }

    pub fn step(&mut self) {
        self.cpu.execute_cycle();
        self.current_scanline = (self.current_scanline + 1) % 192;
    }
}
