use std::path::PathBuf;

use msx::Msx;

#[derive(Debug, Default, Clone)]
pub struct Runner {
    pub rom: PathBuf,
    pub breakpoints: Vec<u16>,
    pub max_cycles: Option<u64>,
    pub open_msx: bool,
    pub break_on_mismatch: bool,
    pub track_flags: bool,

    running: bool,
}

impl Runner {
    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut msx = Msx::new();
        msx.load_binary(self.rom.to_str().unwrap(), 0x0000)?;

        self.running = true;

        let mut stop_next = false;

        loop {
            msx.step();

            let mut stop = !self.running;

            if self.breakpoints.contains(&msx.pc()) {
                println!("Breakpoint hit at {:#06X}", msx.pc());
                stop = true;
            }

            if stop || stop_next {
                if stop_next {
                    println!("Stepped to {:#06X}", msx.pc());
                }
                stop_next = false;
            }

            if msx.halted() {
                break;
            }
        }

        Ok(())
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
        }
    }
}
