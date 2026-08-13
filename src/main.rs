use clap::Parser;
use injm::cli::{self, Cli};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let global_args = cli.global_args;

    match cli.command {
        Some(cli::Commands::Inject(args)) => injm::cmd::inject::run(args, global_args),
        Some(cli::Commands::List(args)) => injm::cmd::list::run(args, global_args),
        Some(cli::Commands::Check(args)) => injm::cmd::check::run(args, global_args),
        None => injm::cmd::root::run(cli.root_args, global_args),
    }
}
