use std::{num::ParseIntError, path::PathBuf};

use anyhow::{anyhow, bail};
use msx::{Msx, ProgramEntry};
use rustyline::DefaultEditor;
use similar::{ChangeTag, TextDiff};

use crate::{
    internal_state::{InternalState, ReportState},
    mru::MRUList,
    open_msx::Client,
};

pub struct Runner {
    pub rom: PathBuf,
    pub breakpoints: Vec<u16>,
    pub max_cycles: Option<u64>,
    pub open_msx: bool,
    pub break_on_mismatch: bool,
    pub track_flags: bool,

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
    Step,

    /// continues execution on all emulators
    Continue,

    /// dumps the current state of all emulators
    Dump,

    /// lists the current loaded program around the current program counter
    List,

    /// lists the execution log
    Log,

    /// adds a breakpoint address
    AddBreakpoint(u16),

    /// removes a breakpoint address
    RemoveBreakpoint(u16),

    /// gets the value of a memory address
    MemGet(u16),

    /// sets the value of a memory address
    MemSet(u16, u8),

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
    fn parse(line: &str) -> anyhow::Result<Self> {
        let mut parts = line.split_whitespace();

        let command = match parts.next() {
            Some("quit") | Some("q") => Command::Quit,
            Some("step") | Some("n") => Command::Step,
            Some("cont") | Some("c") => Command::Continue,
            Some("reset") => Command::Reset,
            Some("list") | Some("l") => Command::List,
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
                let target = match parts.next() {
                    None | Some("diff") => DumpTarget::Diff,
                    Some("openmsx") => DumpTarget::OpenMsx,
                    Some("msx") => DumpTarget::Msx,
                    _ => bail!(
                        "Invalid target for memdump. Use openmsx, msx or leave it empty for msx."
                    ),
                };

                Command::MemDump(target)
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
        self.msx.load_binary(self.rom.to_str().unwrap())?;

        self.client = if self.open_msx {
            Client::start()?;
            let mut client = Client::new(self.rom.clone())?;
            client.init()?;

            Some(client)
        } else {
            None
        };

        self.running = true;

        let mut stop_next = false;

        loop {
            self.step()?;

            let mut stop = !self.running;

            if let Some(client) = &mut self.client {
                if self.break_on_mismatch {
                    let msx_state = format!("{}", self.msx.report_state()?);
                    let open_msx_state = format!("{}", client.report_state()?);

                    if msx_state != open_msx_state {
                        println!("Mismatch at {:#06X}", self.msx.pc());
                        println!("{}", msx_state);
                        println!("{}", open_msx_state);
                        println!();
                        stop = true;
                    }
                }
            }

            if self.at_breakpoint() {
                println!("Breakpoint hit at {:#06X}", self.msx.pc());
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

    pub fn step(&mut self) -> anyhow::Result<()> {
        self.instructions.push(self.msx.instruction());
        self.msx.step();

        if let Some(client) = &mut self.client {
            client.step()?;
        }

        self.cycles += 1;

        Ok(())
    }

    pub fn at_breakpoint(&mut self) -> bool {
        self.breakpoints.contains(&self.msx.pc())
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
        for instruction in self.instructions.iter() {
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
            Command::Step => {
                self.step()?;
                Ok(true)
            }
            Command::Continue => {
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
                            let diff = TextDiff::from_lines(&msx_dump, &openmsx_dump);

                            if !diff.iter_all_changes().any(|c| c.tag() != ChangeTag::Equal) {
                                println!("No differences.");
                                return Ok(true);
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
                                print!("{}{}", sign, change);
                            }
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
    rom: PathBuf,
    breakpoints: Vec<u16>,
    max_cycles: Option<u64>,
    open_msx: bool,
    break_on_mismatch: bool,
    track_flags: bool,
}

impl RunnerBuilder {
    pub fn new(rom: PathBuf) -> Self {
        Self {
            rom,
            breakpoints: Vec::new(),
            max_cycles: None,
            open_msx: false,
            break_on_mismatch: false,
            track_flags: false,
        }
    }

    pub fn with_breakpoints(&mut self, breakpoints: Vec<u16>) -> &mut Self {
        self.breakpoints = breakpoints;
        self
    }

    pub fn with_max_cycles(&mut self, max_cycles: Option<u64>) -> &mut Self {
        self.max_cycles = max_cycles;
        self
    }

    pub fn with_open_msx(&mut self, open_msx: bool) -> &mut Self {
        self.open_msx = open_msx;
        self
    }

    pub fn with_break_on_mismatch(&mut self, break_on_mismatch: bool) -> &mut Self {
        self.break_on_mismatch = break_on_mismatch;
        self
    }

    pub fn with_track_flags(&mut self, track_flags: bool) -> &mut Self {
        self.track_flags = track_flags;
        self
    }

    pub fn build(&self) -> Runner {
        Runner {
            rom: self.rom.clone(),
            breakpoints: self.breakpoints.clone(),
            max_cycles: self.max_cycles,
            open_msx: self.open_msx,
            break_on_mismatch: self.break_on_mismatch,
            track_flags: self.track_flags,
            running: false,
            client: None,
            msx: Msx::new(),
            cycles: 0,
            instructions: MRUList::new(100),
        }
    }
}

impl ReportState for Msx {
    fn report_state(&mut self) -> anyhow::Result<InternalState> {
        let cpu = &self.cpu;
        Ok(InternalState {
            a: cpu.a,
            f: cpu.f,
            b: cpu.b,
            c: cpu.c,
            d: cpu.d,
            e: cpu.e,
            h: cpu.h,
            l: cpu.l,
            sp: cpu.sp,
            pc: cpu.pc,
            hl: cpu.get_hl(),
            hl_contents: cpu.read_byte(cpu.get_hl()),
            opcode: cpu.read_byte(cpu.pc),
        })
    }
}
