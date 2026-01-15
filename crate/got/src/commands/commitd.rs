use crate::cli::CommitdArgs;
use anyhow::{Context, Result};

pub fn execute(args: CommitdArgs) -> Result<()> {
    Err(anyhow::anyhow!("commitd tool not yet implemented")
        .context("Run 'got commitd --help' for usage information"))
}
