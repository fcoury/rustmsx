use std::{num::ParseIntError, path::PathBuf};

use anyhow::{anyhow, bail};
use msx::{
    compare_slices,
    slot::{RamSlot, RomSlot, SlotType},
    Msx, ProgramEntry, ReportState,
};
use rustyline::DefaultEditor;
use similar::{ChangeTag, TextDiff};

use crate::{mru::MRUList, open_msx::Client};

pub struct Runner {
    pub breakpoints: Vec<u16>,
    pub max_cycles: Option<u64>,
    pub open_msx: bool,
    pub break_on_mismatch: bool,
    pub break_on_mem_mismatch: bool,
    pub break_on_ppi_write: bool,
    pub break_on_halt: bool,
    pub log_on_mismatch: bool,
    pub track_flags: bool,
    pub report_every: Option<u64>,

    slots: Vec<SlotType>,
    running: bool,
    cycles: u64,
    client: Option<Client>,
    instructions: MRUList<ProgramEntry>,
    msx: Msx,
}

enum SetTarget {
    A,
    B,
    C,
    HL,
    HLAddress,
}

enum DumpTarget {
    Msx,
    OpenMsx,
    Diff,
}

enum Command {
    /// quits the emulator
    Quit,

    /// resets the emulator at initial state after loading the ROM
    Reset,

    /// steps one instruction on all emulators
    Step(u32),

    /// continues execution on all emulators
    Continue,

    /// dumps the current state of all emulators
    Dump,

    /// lists the current loaded program around the current program counter
    List,

    /// lists the execution log
    Log,

    /// Status
    Status,

    /// adds a breakpoint address
    AddBreakpoint(u16),

    /// removes a breakpoint address
    RemoveBreakpoint(u16),

    /// gets the value of a memory address
    MemGet(u16),

    /// sets the value of a memory address
    MemSet(u16, u8),

    /// dumps vram contents
    VramDump(DumpTarget),

    /// dumps the contents of the memory
    MemDump(DumpTarget),

    /// sets the value of a register
    Set(SetTarget),

    /// sends a command to openMSX
    Send(Vec<String>),
}

struct CommandLine {
    command: Command,
    args: Vec<String>,
}

impl CommandLine {
    fn parse_target(target: Option<&str>) -> anyhow::Result<DumpTarget> {
        match target {
            Some("msx") => Ok(DumpTarget::Msx),
            Some("openmsx") => Ok(DumpTarget::OpenMsx),
            None | Some("diff") => Ok(DumpTarget::Diff),
            _ => bail!("Invalid target. Use openmsx, msx or diff."),
        }
    }

    fn parse(line: &str) -> anyhow::Result<Self> {
        let mut parts = line.split_whitespace();

        let command = match parts.next() {
            Some("quit") | Some("q") => Command::Quit,
            Some("step") | Some("n") => {
                let n = match parts.next() {
                    Some(n) => n.parse()?,
                    None => 1,
                };
                Command::Step(n)
            }
            Some("cont") | Some("c") => Command::Continue,
            Some("reset") => Command::Reset,
            Some("list") | Some("l") => Command::List,
            Some("status") | Some("st") => Command::Status,
            Some("set") | Some("s") => {
                let target = match parts.next() {
                    Some("a") => SetTarget::A,
                    Some("b") => SetTarget::B,
                    Some("c") => SetTarget::C,
                    Some("hl") => SetTarget::HL,
                    Some("(hl)") => SetTarget::HLAddress,
                    _ => panic!("Invalid set target"),
                };

                Command::Set(target)
            }
            Some("dump") | Some("d") => Command::Dump,
            Some("mem") | Some("m") => {
                let addr = u16::from_str_radix(parts.next().unwrap(), 16)?;

                match parts.next() {
                    Some(p) => {
                        let value = u8::from_str_radix(p, 16)?;
                        Command::MemSet(addr, value)
                    }
                    None => Command::MemGet(addr),
                }
            }
            Some("break") | Some("bp") => {
                let addr = u16::from_str_radix(parts.next().unwrap(), 16)?;
                Command::AddBreakpoint(addr)
            }
            Some("removebreak") | Some("rbp") => {
                let addr = u16::from_str_radix(parts.next().unwrap(), 16)?;
                Command::RemoveBreakpoint(addr)
            }
            Some("send") => {
                let mut args = Vec::new();

                for arg in parts.by_ref() {
                    args.push(arg.to_string());
                }

                Command::Send(args)
            }
            Some("memdump") | Some("md") => {
                Command::MemDump(CommandLine::parse_target(parts.next())?)
            }
            Some("vramdump") | Some("vdpdump") | Some("vd") => {
                Command::VramDump(CommandLine::parse_target(parts.next())?)
            }
            Some("log") => Command::Log,
            _ => bail!("Invalid command: {}", line),
        };

        let args = parts.map(|s| s.to_string()).collect();

        Ok(Self { command, args })
    }
}

impl Runner {
    pub fn run(&mut self) -> anyhow::Result<()> {
        self.client = if self.open_msx {
            Client::start()?;
            let mut client = Client::new(&self.slots)?;
            client.init()?;

            Some(client)
        } else {
            None
        };

        self.running = true;

        let mut stop_next = false;

        loop {
            let mut stop = self.step()?;

            if let Some(report_every) = self.report_every {
                if self.cycles % report_every == 0 {
                    println!("\rCycles: {} PC: {:04X}", self.cycles, self.msx.pc());
                    self.dump()?;
                }
            }

            stop = stop || !self.running;

            if let Some(client) = &mut self.client {
                if self.break_on_mismatch || self.log_on_mismatch {
                    let msx_state = format!("{}", self.msx.report_state()?);
                    let open_msx_state = format!("{}", client.report_state()?);

                    if msx_state != open_msx_state {
                        println!("Mismatch at {:#06X}", self.msx.pc());
                        println!("{}", msx_state);
                        println!("{}", open_msx_state);
                        println!();
                        if self.break_on_mismatch {
                            stop = true;
                        }
                    }
                }

                if self.break_on_mem_mismatch {
                    let start = 0u16;
                    let end = (self.msx.mem_size() - 1) as u16;
                    let msx_memory = self.msx.memory();
                    let openmsx_memory = client.memory(start, end)?;

                    if compare_slices(&msx_memory, &openmsx_memory).is_eq() {
                        let msx_dump = self.msx.memory_dump(start, end);
                        let openmsx_dump = client.memory_dump(start, end)?;

                        println!("Memory mismatched at {:#06X}", self.msx.pc());
                        println!();
                        println!("Memory diff from {:#06X} to {:#06X}", start, end);
                        println!("{}", self.diff(msx_dump, openmsx_dump));
                        println!();
                        stop = true;
                    }
                }
            }

            if self.break_on_halt && self.msx.halted() {
                println!("Halted at {:#06X}", self.msx.pc());
                stop = true;
            }

            if self.break_on_ppi_write && self.at_ppi_write() {
                println!("PPI write at {:#06X}", self.msx.pc());
                stop = true;
            }

            if self.at_breakpoint() {
                println!("Breakpoint hit at {:#06X}", self.msx.pc());
                stop = true;
            }

            if self.at_cycles_limit() {
                println!("Breaking at cycle #{}", self.cycles);
                stop = true;
            }

            if stop || stop_next {
                if stop_next {
                    println!("Stepped to {:#06X}", self.msx.pc());
                }
                stop_next = false;

                self.start_prompt()?;
            }

            if self.msx.halted() || !self.running {
                break;
            }
        }

        if let Some(client) = &mut self.client {
            client.shutdown()?;
        }

        Ok(())
    }

    pub fn step(&mut self) -> anyhow::Result<bool> {
        self.instructions.push(self.msx.instruction());
        self.msx.step();

        if let Some(client) = &mut self.client {
            // let opcode = self.msx.cpu.read_byte(self.msx.pc());
            client.step()?;
            // if self.msx.cpu.read_byte(0xFFFF) == 0x00 {
            //     println!(
            //         "OpenMSX halted at {:#06X} with 0xFFFF = 0x00",
            //         self.msx.pc()
            //     );
            //     return Ok(true);
            // }
        }

        self.cycles += 1;

        Ok(false)
    }

    pub fn at_ppi_write(&mut self) -> bool {
        self.msx.wrote_to_ppi()
    }

    pub fn at_breakpoint(&mut self) -> bool {
        self.breakpoints.contains(&self.msx.pc())
    }

    pub fn at_cycles_limit(&mut self) -> bool {
        let is_at = self
            .max_cycles
            .map(|limit| self.cycles >= limit)
            .unwrap_or(false);
        if is_at {
            self.max_cycles = None;
        }
        is_at
    }

    pub fn dump(&mut self) -> anyhow::Result<()> {
        let state = &self.msx.report_state()?;
        println!("{}", state);

        if let Some(client) = &mut self.client {
            let state = client.report_state()?;
            println!("{}", state);
        }

        println!();
        Ok(())
    }

    pub fn list(&mut self) -> anyhow::Result<()> {
        let program = self.msx.program_slice(10, 20);
        for line in program {
            let flag = if self.msx.pc() == line.address {
                ">"
            } else {
                " "
            };
            println!("{} {}", flag, line);
        }

        println!();
        Ok(())
    }

    pub fn log(&mut self) -> anyhow::Result<()> {
        let instructions = self.instructions.iter().collect::<Vec<_>>();
        for instruction in instructions.iter().rev() {
            println!("{}", instruction);
        }

        println!();
        Ok(())
    }

    pub fn start_prompt(&mut self) -> anyhow::Result<()> {
        let history_file = PathBuf::new()
            .join(dirs::home_dir().unwrap())
            .join(".rustmsx_history");

        let mut rl = DefaultEditor::new()?;
        if rl.load_history(&history_file).is_err() {
            println!("No previous history.");
        }

        loop {
            let readline = rl.readline(format!("#{:04X}> ", self.msx.pc()).as_str());

            if let Ok(command) = readline {
                rl.add_history_entry(command.as_str())?;
                if !self.handle_command(command.as_str())? {
                    break;
                }
            }
        }

        rl.append_history(&history_file)?;

        Ok(())
    }

    pub fn handle_command(&mut self, command: &str) -> anyhow::Result<bool> {
        let line = match CommandLine::parse(command) {
            Ok(line) => line,
            Err(e) => {
                println!("{}\n", e);
                return Ok(true);
            }
        };

        match line.command {
            Command::Quit => {
                self.running = false;
                Ok(false)
            }
            Command::Step(n) => {
                for _ in 0..n {
                    self.step()?;
                }
                self.dump()?;
                Ok(true)
            }
            Command::Continue => {
                self.max_cycles = None;
                self.running = true;
                Ok(false)
            }
            Command::Reset => {
                self.msx.reset();
                Ok(true)
            }
            Command::Dump => {
                self.dump()?;
                Ok(true)
            }
            Command::List => {
                self.list()?;
                Ok(true)
            }
            Command::Log => {
                self.log()?;
                Ok(true)
            }
            Command::Status => {
                println!("Cycles: {}", self.cycles);
                println!("Breakpoints: {:?}", self.breakpoints);
                println!(
                    "Primary Slot Config: {:08b}",
                    self.msx.primary_slot_config()
                );
                for (n, slot) in self.slots.iter().enumerate() {
                    println!("Slot #{}: {}", n, slot);
                }
                self.msx
                    .memory_segments()
                    .iter()
                    .enumerate()
                    .for_each(|(n, segment)| {
                        println!("Segment {}: {}", n, segment);
                    });
                self.msx.print_memory_page_info();
                println!();
                Ok(true)
            }
            Command::MemSet(addr, value) => {
                self.msx.set_memory(addr, value);
                Ok(true)
            }
            Command::MemGet(addr) => {
                let value = self.msx.get_memory(addr);
                println!("{:#06X}: {:#04X}", addr, value);
                Ok(true)
            }
            Command::Set(target) => {
                let value = line
                    .args
                    .get(0)
                    .ok_or_else(|| anyhow!("Missing set value"))?;

                match target {
                    SetTarget::A => self.msx.set_a(parse_as_u8(value)?),
                    SetTarget::B => self.msx.set_b(parse_as_u8(value)?),
                    SetTarget::C => self.msx.set_c(parse_as_u8(value)?),
                    SetTarget::HL => self.msx.set_hl(parse_as_u16(value)?),
                    SetTarget::HLAddress => self.msx.set_hl_address(parse_as_u16(value)?),
                }

                Ok(true)
            }
            Command::AddBreakpoint(addr) => {
                self.breakpoints.push(addr);
                Ok(true)
            }
            Command::RemoveBreakpoint(addr) => {
                self.breakpoints.retain(|&a| a != addr);
                Ok(true)
            }
            Command::Send(args) => {
                if let Some(client) = &mut self.client {
                    match client.send(&args.join(" ")) {
                        Ok(_) => {}
                        Err(e) => println!("Error: {}", e),
                    }
                }

                Ok(true)
            }
            Command::VramDump(target) => {
                match target {
                    DumpTarget::Msx => {
                        println!("VRAM dump");
                        println!("{}", self.msx.vram_dump());
                    }
                    DumpTarget::OpenMsx => {
                        if let Some(client) = &mut self.client {
                            println!("VRAM dump");
                            println!("{}", client.vram_dump()?);
                        }
                    }
                    DumpTarget::Diff => {
                        if let Some(client) = &mut self.client {
                            let msx_dump = self.msx.vram_dump();
                            let openmsx_dump = client.vram_dump()?;
                            let diff = self.diff(msx_dump, openmsx_dump);

                            println!("VRAM diff");
                            println!("{}", diff);
                        }
                    }
                }

                println!();
                Ok(true)
            }
            Command::MemDump(target) => {
                let start = 0u16;
                let end = (self.msx.mem_size() - 1) as u16;

                match target {
                    DumpTarget::Msx => {
                        println!("Memory dump from {:#06X} to {:#06X}", start, end);
                        println!("{}", self.msx.memory_dump(start, end));
                    }
                    DumpTarget::OpenMsx => {
                        if let Some(client) = &mut self.client {
                            println!("Memory dump from {:#06X} to {:#06X}", start, end);
                            println!("{}", client.memory_dump(start, end)?);
                        }
                    }
                    DumpTarget::Diff => {
                        if let Some(client) = &mut self.client {
                            let msx_dump = self.msx.memory_dump(start, end);
                            let openmsx_dump = client.memory_dump(start, end)?;

                            println!("Memory diff from {:#06X} to {:#06X}", start, end);
                            println!("{}", self.diff(msx_dump, openmsx_dump));
                        } else {
                            eprintln!("Can't diff memory: no openMSX connection.");
                        }
                    }
                }
                println!();

                Ok(true)
            }
        }
    }

    pub fn diff(&self, msx_dump: String, openmsx_dump: String) -> String {
        let mut res = String::new();
        let diff = TextDiff::from_lines(&msx_dump, &openmsx_dump);

        if !diff.iter_all_changes().any(|c| c.tag() != ChangeTag::Equal) {
            return "No differences.".to_string();
        }

        for change in diff.iter_all_changes() {
            if change.tag() == ChangeTag::Equal {
                continue;
            }
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            res.push_str(&format!("{}{}", sign, change));
        }

        res
    }
}

fn parse_as_u8(s: &str) -> Result<u8, ParseIntError> {
    if let Some(end) = s.strip_prefix("0x") {
        u8::from_str_radix(end, 16)
    } else if s.starts_with('$') || s.starts_with('#') {
        u8::from_str_radix(&s[1..], 16)
    } else {
        s.parse()
    }
}

fn parse_as_u16(s: &str) -> Result<u16, ParseIntError> {
    if let Some(end) = s.strip_prefix("0x") {
        u16::from_str_radix(end, 16)
    } else if s.starts_with('$') || s.starts_with('#') {
        u16::from_str_radix(&s[1..], 16)
    } else {
        s.parse()
    }
}

pub struct RunnerBuilder {
    slots: Vec<SlotType>,
    breakpoints: Vec<u16>,
    max_cycles: Option<u64>,
    open_msx: bool,
    break_on_mismatch: bool,
    break_on_mem_mismatch: bool,
    break_on_ppi_write: bool,
    break_on_halt: bool,
    log_on_mismatch: bool,
    track_flags: bool,
    report_every: Option<u64>,
}

impl RunnerBuilder {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            breakpoints: Vec::new(),
            max_cycles: None,
            open_msx: false,
            break_on_mismatch: false,
            break_on_mem_mismatch: false,
            break_on_ppi_write: false,
            break_on_halt: false,
            log_on_mismatch: false,
            track_flags: false,
            report_every: None,
        }
    }

    pub fn breakpoints(&mut self, breakpoints: Vec<u16>) -> &mut Self {
        self.breakpoints = breakpoints;
        self
    }

    pub fn max_cycles(&mut self, max_cycles: Option<u64>) -> &mut Self {
        self.max_cycles = max_cycles;
        self
    }

    pub fn open_msx(&mut self, open_msx: bool) -> &mut Self {
        self.open_msx = open_msx;
        self
    }

    pub fn break_on_mismatch(&mut self, break_on_mismatch: bool) -> &mut Self {
        self.break_on_mismatch = break_on_mismatch;
        self
    }

    pub fn break_on_mem_mismatch(&mut self, break_on_mem_mismatch: bool) -> &mut Self {
        self.break_on_mem_mismatch = break_on_mem_mismatch;
        self
    }

    pub fn break_on_ppi_write(&mut self, break_on_ppi_write: bool) -> &mut Self {
        self.break_on_ppi_write = break_on_ppi_write;
        self
    }

    pub fn break_on_halt(&mut self, break_on_halt: bool) -> &mut Self {
        self.break_on_halt = break_on_halt;
        self
    }

    pub fn log_on_mismatch(&mut self, log_on_mismatch: bool) -> &mut Self {
        self.log_on_mismatch = log_on_mismatch;
        self
    }

    pub fn track_flags(&mut self, track_flags: bool) -> &mut Self {
        self.track_flags = track_flags;
        self
    }

    pub fn empty_slot(&mut self) -> &mut Self {
        self.slots.push(SlotType::Empty);
        self
    }

    pub fn ram_slot(&mut self, base: u16, size: u32) -> &mut Self {
        self.slots.push(SlotType::Ram(RamSlot::new(base, size)));
        self
    }

    pub fn rom_slot_from_file(
        &mut self,
        rom_path: PathBuf,
        base: u16,
        size: u32,
    ) -> anyhow::Result<&mut Self> {
        self.slots
            .push(SlotType::Rom(RomSlot::load(rom_path, base, size)?));
        Ok(self)
    }

    pub fn report_every(&mut self, n_cycles: Option<u64>) -> &mut Self {
        self.report_every = n_cycles;
        self
    }

    pub fn build(&self) -> Runner {
        Runner {
            slots: self.slots.clone(),
            breakpoints: self.breakpoints.clone(),
            max_cycles: self.max_cycles,
            open_msx: self.open_msx,
            break_on_mismatch: self.break_on_mismatch,
            break_on_mem_mismatch: self.break_on_mem_mismatch,
            break_on_ppi_write: self.break_on_ppi_write,
            break_on_halt: self.break_on_halt,
            log_on_mismatch: self.log_on_mismatch,
            track_flags: self.track_flags,
            report_every: self.report_every,
            running: false,
            client: None,
            msx: Msx::new(&self.slots),
            cycles: 0,
            instructions: MRUList::new(100),
        }
    }
}
