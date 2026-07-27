use anyhow::Result;
use std::fs;
use std::io::{self, Read};

use crate::cli::{GlobalArgs, InjectArgs};
use crate::config::load_config;
use crate::injector::inject;
use crate::output::print_diff;
use crate::parser::{PatternParserOption, parse_patterns};
use crate::types::{BlockRole, MarkerBlock, SourceSpan};
use crate::validator::{validate_duplicated_input_ids, validate_missing_ids};

pub fn run(args: InjectArgs, global_args: GlobalArgs) -> Result<()> {
    let cfg = load_config(global_args.config)?;

    let excludes: Vec<String> = cfg.exclude.into_iter().chain(global_args.exclude).collect();
    let pattern_parser_opts = PatternParserOption {
        no_gitignore: global_args.no_gitignore,
        cwd: std::env::current_dir()?,
    };

    let output_patterns: Vec<String> = args.output.into_iter().chain(cfg.output).collect();
    let output_files = parse_patterns(&output_patterns, &excludes, &pattern_parser_opts)?;

    let input_blocks: Vec<MarkerBlock> = if args.input.is_empty() {
        stdin_blocks(args.id)?
    } else {
        let input_patterns: Vec<String> = args.input.into_iter().chain(cfg.input).collect();
        let input_files = parse_patterns(&input_patterns, &excludes, &pattern_parser_opts)?;
        validate_missing_ids(&output_files, &input_files)?;
        input_files
            .into_iter()
            .flat_map(|file| file.blocks)
            .collect()
    };

    validate_duplicated_input_ids(&input_blocks)?;

    let multiple_outputs = output_files.len() > 1;
    for output_file in output_files {
        let replaced = inject(&output_file.content, &output_file.blocks, &input_blocks)?;
        if args.dry_run {
            if args.diff {
                print_diff(&output_file.path, &output_file.content, &replaced);
                continue;
            }

            if multiple_outputs {
                println!("==== {} ====", output_file.path.display());
            }
            println!("{replaced}");
        } else {
            fs::write(output_file.path, replaced)?;
        }
    }

    Ok(())
}

fn read_stdin() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

fn stdin_blocks(ids: Vec<Option<String>>) -> Result<Vec<MarkerBlock>> {
    let stdin = read_stdin()?;
    let input_ids = ids.into_iter().flatten().collect();
    Ok(vec![MarkerBlock {
        span: SourceSpan::new(0, 0),
        role: BlockRole::Input { ids: input_ids },
        content: stdin,
    }])
}
