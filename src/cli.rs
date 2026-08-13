use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::output::OutputFormat;

#[derive(Parser)]
#[command(
    name = "injm",
    about = "Inject stdin content into marked regions between `injm begin` and `injm end` comments.",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub global_args: GlobalArgs,

    #[command(flatten)]
    pub root_args: RootArgs,
}

#[derive(Args)]
pub struct RootArgs {
    /// Preview changes
    #[arg(long)]
    pub dry_run: bool,

    /// Print diff of old and new contents
    #[arg(long)]
    pub diff: bool,
}

#[derive(Args)]
pub struct GlobalArgs {
    /// Path to configuration file [default: injm.toml]
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Files to be excluded (support glob)
    #[arg(short, long, global = true, num_args = 1..)]
    pub exclude: Vec<String>,

    /// Don't exclude files in `.gitignore`
    #[arg(long, global = true)]
    pub no_gitignore: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Inject stdin or input blocks into output files
    Inject(InjectArgs),

    /// List marker blocks
    List(ListArgs),

    /// Check whether marker blocks are synchronized
    Check(CheckArgs),
}

#[derive(Args)]
pub struct InjectArgs {
    /// Files contains input blocks (optional)
    #[arg(short, long, num_args = 1..)]
    pub input: Vec<String>,

    /// Files to be injected
    #[arg(short, long, required = true, num_args = 1..)]
    pub output: Vec<String>,

    /// Preview injected results
    #[arg(long)]
    pub dry_run: bool,

    /// Print diff between original and injected files
    #[arg(long)]
    pub diff: bool,

    /// Specify output blocks to be injected (optional)
    #[arg(long, num_args = 1..)]
    pub id: Vec<Option<String>>,
}

#[derive(Args)]
pub struct ListArgs {
    /// Files to list marker blocks (support glob)
    #[arg(num_args = 1..)]
    pub files: Vec<String>,

    /// Output format
    #[arg(long, short, default_value = "table")]
    pub format: OutputFormat,
}

#[derive(Args)]
pub struct CheckArgs {
    /// Files to be checked (support glob)
    #[arg(num_args = 1..)]
    pub files: Vec<String>,

    /// Print diff between expected and not synchronized blocks
    #[arg(long)]
    pub diff: bool,
}
