use crate::cli::GotselArgs;
use anyhow::{Context, Result};

pub fn execute(_args: GotselArgs) -> Result<()> {
    Err(anyhow::anyhow!("gotsel tool not yet implemented")
        .context("Run 'got gotsel --help' for usage information"))
}
