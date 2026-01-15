use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "got",
    about = "A toolkit for git tools your past self should have used",
    version,
    long_about = None,
    color = clap::ColorChoice::Auto,
)]
pub struct Cli {
    /// Global configuration file
    #[arg(short, long, global = true, env = "GOT_CONFIG")]
    pub config: Option<std::path::PathBuf>,

    /// Output format
    #[arg(
        short,
        long,
        global = true,
        value_enum,
        default_value = "auto",
        env = "GOT_OUTPUT"
    )]
    pub output: OutputFormat,

    /// Increase logging verbosity
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress all output
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum OutputFormat {
    #[default]
    Auto,
    Plain,
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Find the oldest changes and get a datestamp for that file
    Goldest(GoldestArgs),

    /// Git staging selection tree tool
    Gotsel(GotselArgs),

    /// Status focused on modified time
    Statusd(StatusdArgs),

    /// Commit using the most recent date of what is staged
    Commitd(CommitdArgs),

    /// Ignore files
    Nah {
        #[command(subcommand)]
        command: NahArgs,
    },
}

#[derive(Args, Debug)]
pub struct GoldestArgs {
    /// Return only file path
    #[arg(short, long, conflicts_with_all = ["date_only", "short", "porcelain"])]
    pub file_only: bool,

    /// Return only date
    #[arg(short = 'd', long, conflicts_with_all = ["file_only", "short", "porcelain"])]
    pub date_only: bool,

    /// Filter equivalent to git status -u filtering
    #[arg(short = 'u', value_name = "FILTER")]
    pub untracked: Option<String>,

    /// Number of results to show
    #[arg(short = 'l', long, value_name = "LINES", default_value = "1")]
    pub lines: usize,

    /// Skip N results
    #[arg(short = 's', long, value_name = "SKIP", default_value = "0")]
    pub skip: usize,

    /// Show git status --short output format
    #[arg(short, long, conflicts_with_all = ["file_only", "date_only", "porcelain"])]
    pub short: bool,

    /// Show git status --porcelain output format
    #[arg(long, conflicts_with_all = ["file_only", "date_only", "short"])]
    pub porcelain: bool,
}

#[derive(Args, Debug)]
pub struct GotselArgs {
    /// Path to git repository
    #[arg(short, long)]
    pub repo: Option<std::path::PathBuf>,

    /// Start with files staged
    #[arg(short, long)]
    pub staged: bool,
}

#[derive(Args, Debug)]
pub struct StatusdArgs {
    /// Show untracked files
    #[arg(short = 'u', long)]
    pub untracked: bool,

    /// Show untracked files in subdirectories
    #[arg(long)]
    pub untracked_all: bool,
}

#[derive(Args, Debug)]
pub struct CommitdArgs {
    /// Commit message
    #[arg(short, long, value_name = "MSG")]
    pub message: Option<String>,

    /// Allow empty commit
    #[arg(long)]
    pub allow_empty: bool,

    /// Dry run - show what would be committed
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}

#[derive(Subcommand, Debug)]
pub enum NahArgs {
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
}
