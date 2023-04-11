mod mru;
mod open_msx;
mod runner;

use std::path::PathBuf;

use clap::Parser;
use runner::RunnerBuilder;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Parser, Debug)]
pub struct Cli {
    /// Path to the complete ROM file
    rom_path: PathBuf,

    /// Maximum number of cycles to run before breaking
    #[clap(short = 'c', long)]
    max_cycles: Option<u64>,

    /// Track flag changes
    #[clap(short, long)]
    track_flags: bool,

    /// Break on the given address(es)
    #[clap(short, long)]
    breakpoint: Vec<String>,

    /// Runs openMSX in paralell
    #[clap(short, long)]
    open_msx: bool,

    /// Break on CPU registers and flags mismatch between openMSX and emulator
    #[clap(short = 'm', long)]
    break_on_mismatch: bool,

    /// Break on memory mismatch between openMSX and emulator
    #[clap(short = 'e', long)]
    break_on_mem_mismatch: bool,

    /// Break on HALT instruction
    #[clap(long)]
    break_on_halt: bool,

    /// Dump a log on mismatch between openMSX and emulator
    #[clap(short, long)]
    log_on_mismatch: bool,

    /// Dump a log every n cycles
    #[clap(short, long)]
    report_every: Option<u64>,

    /// Break on PPI write operations
    #[clap(short = 'p', long)]
    break_on_ppi_write: bool,

    /// Enable debug logging
    #[clap(short, long)]
    debug: bool,

    /// Enable debug logging for the VDP
    #[clap(long)]
    debug_vdp: bool,

    /// Enable debug logging for the PPI
    #[clap(long)]
    debug_ppi: bool,
}

pub fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_level = format!(
        "msx_emulator={},msx::cpu=error,msx::vdp={},msx::ppi={},info",
        if cli.debug { "trace" } else { "info" },
        if cli.debug_vdp { "trace" } else { "error" },
        if cli.debug_ppi { "trace" } else { "error" },
    );
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(log_level))?,
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut runner = RunnerBuilder::new()
        .rom_slot_from_file(cli.rom_path, 0x0000, 0x10000)?
        // .ram_slot(0x0000, 0xFFFF)
        // .ram_slot(0x0000, 0xFFFF)
        .empty_slot()
        .empty_slot()
        .ram_slot(0x0000, 0x10000)
        .max_cycles(cli.max_cycles)
        .track_flags(cli.track_flags)
        .breakpoints(
            cli.breakpoint
                .iter()
                .map(|s| u16::from_str_radix(s, 16).unwrap())
                .collect(),
        )
        .open_msx(cli.open_msx)
        .break_on_mismatch(cli.break_on_mismatch)
        .log_on_mismatch(cli.log_on_mismatch)
        .break_on_mem_mismatch(cli.break_on_mem_mismatch)
        .break_on_ppi_write(cli.break_on_ppi_write)
        .break_on_halt(cli.break_on_halt)
        .report_every(cli.report_every)
        .build();
    runner.run()?;

    Ok(())
}
