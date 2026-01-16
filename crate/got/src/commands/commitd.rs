use crate::cli::CommitdArgs;
use anyhow::Result;

pub fn execute(_args: CommitdArgs) -> Result<()> {
    const ALIAS: &str = include_str!("../../aliases/alias-commitd.conf");
    gotconfig::ensure_git_alias("commitd", ALIAS)?;
    println!("Added git alias: commitd");
    Ok(())
}
