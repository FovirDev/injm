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
}

#[derive(Args)]
pub struct GlobalArgs {
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    #[arg(short, long, global = true)]
    pub exclude: Vec<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    Inject(InjectArgs),
    List(ListArgs),
    Check(CheckArgs),
}

#[derive(Args)]
pub struct InjectArgs {
    #[arg(short, long, num_args = 1..)]
    pub input: Vec<String>,

    #[arg(short, long, required = true, num_args = 1..)]
    pub output: Vec<String>,

    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub diff: bool,

    #[arg(long)]
    pub id: Vec<Option<String>>,
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(num_args = 1..)]
    pub input: Vec<String>,

    #[arg(long, short, default_value = "table")]
    pub format: OutputFormat,
}

#[derive(Args)]
pub struct CheckArgs {
    #[arg(num_args = 1..)]
    pub files: Vec<String>,

    #[arg(long)]
    pub diff: bool,
}
