mod internal_state;
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

    #[clap(short, long)]
    debug: bool,
}

pub fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_level = format!(
        "msx_emulator={},msx::cpu=error,msx::vdp=error,msx::ppi=error,info",
        if cli.debug { "trace" } else { "info" }
    );
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(log_level))?,
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut runner = RunnerBuilder::new(cli.rom_path)
        .with_max_cycles(cli.max_cycles)
        .with_track_flags(cli.track_flags)
        .with_breakpoints(
            cli.breakpoint
                .iter()
                .map(|s| u16::from_str_radix(s, 16).unwrap())
                .collect(),
        )
        .with_open_msx(cli.open_msx)
        .with_break_on_mismatch(cli.break_on_mismatch)
        .build();
    runner.run()?;

    Ok(())
}
