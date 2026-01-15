use super::*;
use crate::cli::Commands;
use anyhow::Result;

pub fn execute(command: Commands) -> Result<()> {
    match command {
        Commands::Goldest(args) => goldest::execute(args),
        Commands::Gotsel(args) => gotsel::execute(args),
        Commands::Statusd(args) => statusd::execute(args),
        Commands::Commitd(args) => commitd::execute(args),
        Commands::Nah(args) => nah::execute(args),
    }
}
