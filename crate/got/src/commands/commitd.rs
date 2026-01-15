use crate::cli::CommitdArgs;
use crate::util;
use anyhow::{Context, Result};

pub fn execute(_args: CommitdArgs) -> Result<()> {
    util::ensure_git_alias("commitd", "got commitd")?;
    println!("Added git alias: commitd -> got commitd");
    Ok(())
}
