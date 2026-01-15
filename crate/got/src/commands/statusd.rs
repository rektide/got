use crate::cli::StatusdArgs;
use anyhow::{Context, Result};

pub fn execute(args: StatusdArgs) -> Result<()> {
    Err(anyhow::anyhow!("statusd tool not yet implemented")
        .context("Run 'got statusd --help' for usage information"))
}
