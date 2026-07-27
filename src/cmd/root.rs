use std::fs;

use crate::{
    cli::{GlobalArgs, InjectArgs},
    cmd::inject,
    config::load_config,
};
use anyhow::{Result, anyhow};

pub fn run(mut args: GlobalArgs) -> Result<()> {
    let config_path = match &args.config {
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

    args.config = config_path.clone();
    let cfg = load_config(config_path)?;

    if cfg.input.is_empty() {
        return Err(anyhow!("input source is empty"));
    }

    inject::run(
        InjectArgs {
            input: cfg.input,
            output: cfg.output,
            dry_run: false,
            diff: false,
            id: vec![],
        },
        args,
    )?;

    Ok(())
}
