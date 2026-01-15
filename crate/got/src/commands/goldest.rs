use crate::cli::GoldestArgs;
use anyhow::{Context, Result};

pub fn execute(_args: GoldestArgs) -> Result<()> {
    Err(anyhow::anyhow!("goldest tool not yet implemented")
        .context("Run 'got goldest --help' for usage information"))
}
