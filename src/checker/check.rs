use std::path::PathBuf;

use crate::{
    checker::Result,
    injector::inject_into_a_block,
    types::{BlockRole, MarkerBlock, ParsedFile, SourceSpan},
};

pub struct SyncIssue {
    pub path: PathBuf,
    pub span: SourceSpan,
    pub id: String,
    pub expected: String,
    pub actual: String,
}

pub(crate) fn check_sync(
    input_blocks: &[&MarkerBlock],
    output_files: &[ParsedFile],
) -> Result<Vec<SyncIssue>> {
    let mut issues = Vec::new();

    for output_file in output_files {
        let lines: Vec<String> = output_file.content.lines().map(str::to_owned).collect();

        for output_block in &output_file.blocks {
            let BlockRole::Output { id: Some(id) } = &output_block.role else {
                continue;
            };

            let Some(input_block) = input_blocks
                .iter()
                .find(|input| input.matches_output(output_block))
            else {
                continue;
            };

            let injected = inject_into_a_block(&lines, output_block, &input_block.content)?;
            let content_start = output_block.span.begin_marker + 1;
            let content_end = content_start + 2 * output_block.config.offset + 1;

            let expected = injected[content_start..content_end].join("\n");
            if expected != output_block.content {
                issues.push(SyncIssue {
                    path: output_file.path.clone(),
                    span: output_block.span,
                    id: id.clone(),
                    expected,
                    actual: output_block.content.clone(),
                });
            }
        }
    }

    Ok(issues)
}
