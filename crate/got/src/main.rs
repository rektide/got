use anyhow::Result;
use clap::Parser;
use got::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::try_parse().map_err(|e| anyhow::anyhow!("Failed to parse arguments: {}", e))?;

    // Set up logging based on verbosity
    setup_logging(cli.verbose);

    // Execute command
    got::commands::dispatch::execute(cli.command)
}

fn setup_logging(verbosity: u8) {
    use std::env;

    // Set RUST_LOG based on verbosity level
    let log_level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    if env::var("RUST_LOG").is_ok() {
        return;
    }

    env::set_var("RUST_LOG", log_level);
}
