use crate::cli::NahArgs;
use anyhow::{Context, Result};

pub fn execute(args: NahArgs) -> Result<()> {
    match args {
        NahArgs::Add { pattern, global } => execute_add(pattern, global),
        NahArgs::Remove { pattern, global } => execute_remove(pattern, global),
        NahArgs::List { global } => execute_list(global),
        NahArgs::Show { global } => execute_show(global),
    }
}

fn execute_add(pattern: String, global: bool) -> Result<()> {
    Err(anyhow::anyhow!(
        "nah add not yet implemented for: {} (global: {})",
        pattern,
        global
    )
    .context("Run 'got nah add --help' for usage information"))
}

fn execute_remove(pattern: String, global: bool) -> Result<()> {
    Err(anyhow::anyhow!(
        "nah remove not yet implemented for: {} (global: {})",
        pattern,
        global
    )
    .context("Run 'got nah remove --help' for usage information"))
}

fn execute_list(global: bool) -> Result<()> {
    Err(
        anyhow::anyhow!("nah list not yet implemented (global: {})", global)
            .context("Run 'got nah list --help' for usage information"),
    )
}

fn execute_show(global: bool) -> Result<()> {
    Err(
        anyhow::anyhow!("nah show not yet implemented (global: {})", global)
            .context("Run 'got nah show --help' for usage information"),
    )
}
