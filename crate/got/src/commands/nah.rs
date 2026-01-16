use crate::cli::NahArgs;
use anyhow::{Context, Result};
use nah::NahCommands;

pub fn execute(args: NahArgs) -> Result<()> {
    let command = match args {
        NahArgs::Add { pattern, global } => NahCommands::Add { pattern, global },
        NahArgs::Remove { pattern, global } => NahCommands::Remove { pattern, global },
        NahArgs::List { global } => NahCommands::List { global },
        NahArgs::Show { global } => NahCommands::Show { global },
    };

    nah::execute(command)
}
