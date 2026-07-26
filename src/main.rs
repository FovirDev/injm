mod checker;
mod cli;
mod cmd;
mod config;
mod injector;
mod output;
mod parser;
mod types;
mod validator;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    let global_args = cli.global_args;

    match cli.command {
        Some(cli::Commands::Inject(args)) => cmd::inject::run(args, global_args),
        Some(cli::Commands::List(args)) => cmd::list::run(args, global_args),
        Some(cli::Commands::Check(args)) => cmd::check::run(args, global_args),
        None => cmd::root::run(global_args),
    }
}
