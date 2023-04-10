mod mru;
mod open_msx;
mod runner;

use std::path::PathBuf;

use clap::Parser;
use runner::RunnerBuilder;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Parser, Debug)]
pub struct Cli {
    rom_path: PathBuf,

    #[clap(short = 'c', long)]
    max_cycles: Option<u64>,

    #[clap(short, long)]
    track_flags: bool,

    #[clap(short, long)]
    breakpoint: Vec<String>,

    #[clap(short, long)]
    open_msx: bool,

    #[clap(short = 'm', long)]
    break_on_mismatch: bool,

    #[clap(short = 'e', long)]
    break_on_mem_mismatch: bool,

    #[clap(short, long)]
    log_on_mismatch: bool,

    #[clap(short, long)]
    report_every: Option<u64>,

    #[clap(short = 'p', long)]
    break_on_ppi_write: bool,

    #[clap(short, long)]
    debug: bool,

    #[clap(long)]
    debug_vdp: bool,

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
        .rom_slot_from_file(cli.rom_path, 0x0000, 0x8000)?
        .empty_slot()
        .empty_slot()
        .ram_slot(0x0000, 0xFFFF)
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
        .report_every(cli.report_every)
        .build();
    runner.run()?;

    Ok(())
}
