use crate::cli::HookGotArgs;
use anyhow::Result;

pub fn execute(_args: HookGotArgs) -> Result<()> {
    const HOOK: &str = include_str!("../../hooks/hook-zsh.conf");
    println!("{}", HOOK);
    Ok(())
}
