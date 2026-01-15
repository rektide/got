use crate::cli::GoldestArgs;
use anyhow::{Context, Result};

pub fn execute(args: GoldestArgs) -> Result<()> {
    // Placeholder implementation
    if args.file_only {
        anyhow::bail!("goldest --file-only not yet implemented");
    }

    if args.date_only {
        anyhow::bail!("goldest --date-only not yet implemented");
    }

    anyhow::bail!("goldest tool not yet implemented")
        .context("Run 'got goldest --help' for usage information");
}
