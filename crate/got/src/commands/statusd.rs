use crate::cli::StatusdArgs;
use anyhow::Result;

pub fn execute(_args: StatusdArgs) -> Result<()> {
    const ALIAS: &str = include_str!("../../aliases/alias-statusd.conf");
    gotconfig::ensure_git_alias("statusd", ALIAS)?;
    println!("Added git alias: statusd");
    Ok(())
}
