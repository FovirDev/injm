use crate::{
    checker::check_sync,
    cli::{CheckArgs, GlobalArgs},
    output::print_block_diff,
    parser::parse_patterns,
    types::{BlockRole, MarkerBlock},
    validator::{validate_duplicated_input_ids, validate_missing_ids},
};
use anyhow::{Result, bail};

pub fn run(args: CheckArgs, _global_args: GlobalArgs) -> Result<()> {
    let pattern = if args.files.is_empty() {
        vec![".".to_string()]
    } else {
        args.files
    };

    let files = parse_patterns(&pattern)?;

    validate_missing_ids(&files, &files)?;

    let input_blocks: Vec<&MarkerBlock> = files
        .iter()
        .flat_map(|file| file.blocks.iter())
        .filter(|block| matches!(&block.role, BlockRole::Input { .. }))
        .collect();

    validate_duplicated_input_ids(input_blocks.iter().copied())?;

    let issues = check_sync(&input_blocks, &files);
    if issues.is_empty() {
        println!("all marker blocks are synchronized");
        return Ok(());
    }

    for issue in &issues {
        eprintln!(
            "{}:{}: output block `{}` is out of sync",
            issue.path.display(),
            issue.span.display_lines(),
            issue.id
        );

        if args.diff {
            print_block_diff(
                &issue.path,
                &issue.span.display_lines(),
                &issue.id,
                &issue.actual,
                &issue.expected,
            );
        }
    }

    bail!("{} output block(s) are out of sync", issues.len());
}
