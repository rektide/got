use crate::cli::StatusdArgs;
use crate::util;
use anyhow::{Context, Result};

pub fn execute(_args: StatusdArgs) -> Result<()> {
    util::ensure_git_alias("statusd", "got statusd")?;
    println!("Added git alias: statusd -> got statusd");
    Ok(())
}
