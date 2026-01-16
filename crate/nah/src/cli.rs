use crate::{ignore, pick};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "nah",
    about = "File ignore management",
    version,
    long_about = None,
)]
pub struct NahCli {
    #[command(subcommand)]
    pub command: NahCommands,
}

#[derive(Subcommand)]
pub enum NahCommands {
    /// Add a file to ignore list
    Add {
        /// File or pattern to ignore
        #[arg(required = true)]
        pattern: String,

        /// Ignore globally (vs per-repository)
        #[arg(short, long)]
        global: bool,
    },

    /// Remove a file from ignore list
    Remove {
        /// File or pattern to remove
        #[arg(required = true)]
        pattern: String,

        /// Remove from global ignore list
        #[arg(short, long)]
        global: bool,
    },

    /// List ignored files
    List {
        /// Show global ignores
        #[arg(short, long)]
        global: bool,
    },

    /// Show ignore list location
    Show {
        /// Show global ignore location
        #[arg(short, long)]
        global: bool,
    },

    /// Pick files to ignore from untracked files
    Pick {
        /// Add to global ignore list
        #[arg(short, long)]
        global: bool,
    },
}

pub fn execute(cmd: NahCommands) -> Result<()> {
    match cmd {
        NahCommands::Add { pattern, global } => execute_add(&pattern, global),
        NahCommands::Remove { pattern, global } => execute_remove(&pattern, global),
        NahCommands::List { global } => execute_list(global),
        NahCommands::Show { global } => execute_show(global),
        NahCommands::Pick { global } => execute_pick(global),
    }
}

fn execute_add(pattern: &str, global: bool) -> Result<()> {
    ignore::add_pattern(pattern, global, None)?;
    println!(
        "Added '{}' to {}ignore list",
        pattern,
        if global { "global " } else { "" }
    );
    Ok(())
}

fn execute_remove(pattern: &str, global: bool) -> Result<()> {
    ignore::remove_pattern(pattern, global, None)?;
    println!(
        "Removed '{}' from {}ignore list",
        pattern,
        if global { "global " } else { "" }
    );
    Ok(())
}

fn execute_list(global: bool) -> Result<()> {
    let patterns = ignore::list_patterns(global, None)?;

    if patterns.is_empty() {
        println!(
            "No patterns in {}ignore list",
            if global { "global " } else { "" }
        );
    } else {
        for pattern in patterns {
            println!("{}", pattern);
        }
    }

    Ok(())
}

fn execute_show(global: bool) -> Result<()> {
    let path = ignore::get_nah_path(global, None)?;
    println!("{}", path.display());
    Ok(())
}

fn execute_pick(global: bool) -> Result<()> {
    let files = pick::get_untracked_files()?;

    if files.is_empty() {
        println!("No untracked files to ignore.");
        return Ok(());
    }

    pick::display_file_selection(&files)?;
    println!();
    println!("Enter selection (or 'q' to cancel): ");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input == "q" || input.is_empty() {
        println!("Cancelled.");
        return Ok(());
    }

    let selected = pick::parse_selection(input, files.len());

    if selected.is_empty() {
        println!("No valid selection.");
        return Ok(());
    }

    println!();
    for index in selected {
        let pattern = &files[index - 1];
        ignore::add_pattern(pattern, global, None)?;
        println!("Ignored: {}", pattern);
    }

    Ok(())
}
