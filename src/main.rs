//! Thin CLI entry point: parses arguments and delegates to the library pipeline in
//! `src/lib.rs`, per `specs/001-remote-movie-catalog/contracts/cli-interface.md`.

use std::process::ExitCode;

use clap::Parser;
use jellyfin_catalog_export::cli::Cli;

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            eprint!("{e}");
            return ExitCode::from(jellyfin_catalog_export::EXIT_CONFIG_ERROR as u8);
        }
    };

    ExitCode::from(jellyfin_catalog_export::run(&cli) as u8)
}
