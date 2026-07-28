use std::fs;

use crate::{
    cli::{GlobalArgs, InjectArgs, RootArgs},
    cmd::inject,
    config::load_config,
};
use anyhow::{Result, anyhow};

pub fn run(args: RootArgs, mut global_args: GlobalArgs) -> Result<()> {
    let config_path = match &global_args.config {
        Some(cfg) => Some(cfg.to_owned()),
        None => {
            let cwd = std::env::current_dir()?;
            let path = cwd.join("injm.toml");
            if fs::exists(&path)? {
                Some(path)
            } else {
                return Err(anyhow!("config file is missing"));
            }
        }
    };

    global_args.config = config_path.clone();
    let cfg = load_config(config_path)?;

    if cfg.input.is_empty() {
        return Err(anyhow!("input source is empty"));
    }

    inject::run(
        InjectArgs {
            input: cfg.input,
            output: cfg.output,
            dry_run: args.dry_run,
            diff: args.diff,
            id: vec![],
        },
        global_args,
    )?;

    Ok(())
}
