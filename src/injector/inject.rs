use super::{InjectorError, Result};
use crate::types::MarkerBlock;

pub fn inject(
    content: &str,
    output_blocks: &[MarkerBlock],
    input_blocks: &[MarkerBlock],
) -> Result<String> {
    let mut lines: Vec<String> = content.lines().map(str::to_owned).collect();

    // Use reversed iteration to avoid changes of line number.
    for block in output_blocks.iter().rev() {
        if let Some(input_block) = input_blocks.iter().find(|b| b.matches_output(block)) {
            if input_block.content.is_empty() {
                return Err(InjectorError::EmptyInputContent);
            }
            lines = inject_into_a_block(&lines, block, &input_block.content)?;
        }
    }

    let mut result = lines.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }

    Ok(result)
}

fn inject_into_a_block(lines: &[String], block: &MarkerBlock, stdin: &str) -> Result<Vec<String>> {
    let before_range = block.span.before_lines(block.config.offset);
    let after_range = block.span.after_lines(block.config.offset);

    if before_range.end >= after_range.start {
        // Convert to human-readable error messages.
        return Err(InjectorError::InvalidRange {
            begin: before_range.end + 1,
            end: after_range.start + 1,
        });
    }

    // Content to be replaced is in (before_range, aftger_range).
    let before = &lines[before_range];
    let after = &lines[after_range];

    let stdin = if block.config.trim {
        trim_blank_lines(stdin)
    } else {
        stdin
    };

    let stdin = if let Some(indent) = block.config.indent {
        align_indent(stdin, indent)?
    } else {
        stdin.to_owned()
    };

    let mut injected = Vec::with_capacity(before.len() + after.len() + 1);
    injected.extend_from_slice(before);
    injected.push(stdin);
    injected.extend_from_slice(after);

    Ok(injected)
}

fn trim_blank_lines(content: &str) -> &str {
    let mut s = content;

    while let Some((line, rest)) = s.split_once('\n') {
        if !line.trim().is_empty() {
            break;
        }
        s = rest
    }

    while let Some((rest, line)) = s.rsplit_once('\n') {
        if !line.trim().is_empty() {
            break;
        }
        s = rest;
    }

    let trimmed = s.strip_suffix('\r').unwrap_or(s);
    if trimmed.trim().is_empty() {
        ""
    } else {
        trimmed
    }
}

fn align_indent(content: &str, target_indent_width: usize) -> Result<String> {
    #[derive(PartialEq, Eq)]
    enum IndentChar {
        Space,
        Tab,
    }

    impl IndentChar {
        fn as_char(&self) -> char {
            match self {
                IndentChar::Space => ' ',
                IndentChar::Tab => '\t',
            }
        }
    }

    let mut min_indent_width = usize::MAX;
    let mut indent_char: Option<IndentChar> = None;

    for (line_number, line) in content.split('\n').enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let indents = line.chars().take_while(|c| matches!(c, '\t' | ' '));
        let mut width = 0;

        for c in indents {
            let current_char = match c {
                ' ' => IndentChar::Space,
                '\t' => IndentChar::Tab,
                _ => unreachable!(),
            };

            if let Some(expected_char) = indent_char.as_ref() {
                if *expected_char != current_char {
                    return Err(InjectorError::MixedIndentChar {
                        line_number: line_number + 1,
                    });
                }
            } else {
                indent_char = Some(current_char);
            }

            width += 1;
        }

        min_indent_width = min_indent_width.min(width);
    }

    if min_indent_width == usize::MAX {
        return Ok(content.to_owned());
    }

    let indent_char = indent_char.unwrap_or(IndentChar::Space).as_char();

    let aligned = content
        .split('\n')
        .map(|line| {
            if line.trim().is_empty() {
                return line.to_owned();
            }

            let current_indent_width = line.chars().take_while(|c| matches!(c, ' ' | '\t')).count();
            let new_indent_width = target_indent_width + (current_indent_width - min_indent_width);

            let mut aligned_line = String::with_capacity(new_indent_width + line.len());
            aligned_line.extend(std::iter::repeat_n(indent_char, new_indent_width));
            aligned_line.push_str(&line[current_indent_width..]);
            aligned_line
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(aligned)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::types::{BlockRole, MarkerBlock, MarkerConfig, SourceSpan};

    fn make_default_input_blocks(s: &str) -> Vec<MarkerBlock> {
        vec![MarkerBlock {
            span: SourceSpan::new(0, 0),
            content: s.to_string(),
            role: BlockRole::Input { ids: vec![] },
            config: MarkerConfig::default(),
        }]
    }

    #[test]
    fn test_inject_single_block() {
        let content = "fn main() {\n    // injm begin\n    old content\n    // injm end\n}\n";
        let blocks = vec![MarkerBlock {
            span: SourceSpan::new(1, 3),
            role: BlockRole::Output { id: None },
            content: "".to_string(),
            config: MarkerConfig::default(),
        }];
        let result = inject(
            content,
            &blocks,
            &make_default_input_blocks("    new content"),
        )
        .unwrap();
        assert!(result.contains("new content"));
        assert!(!result.contains("old content"));
    }

    #[test]
    fn test_inject_preserves_markers() {
        let content = "// injm begin\nold\n// injm end\n";
        let blocks = vec![MarkerBlock {
            span: SourceSpan::new(0, 2),
            role: BlockRole::Output { id: None },
            content: "".to_string(),
            config: MarkerConfig::default(),
        }];
        let result = inject(content, &blocks, &make_default_input_blocks("new")).unwrap();
        assert!(result.contains("// injm begin"));
        assert!(result.contains("// injm end"));
    }

    #[test]
    fn test_inject_preserves_trailing_newline() {
        let content = "// injm begin\nold\n// injm end\n";
        let blocks = vec![MarkerBlock {
            span: SourceSpan::new(0, 2),
            role: BlockRole::Output { id: None },
            content: "".to_string(),
            config: MarkerConfig::default(),
        }];
        let result = inject(content, &blocks, &make_default_input_blocks("new")).unwrap();
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_inject_no_trailing_newline() {
        let content = "// injm begin\nold\n// injm end";
        let blocks = vec![MarkerBlock {
            span: SourceSpan::new(0, 2),
            role: BlockRole::Output { id: None },
            content: "".to_string(),
            config: MarkerConfig::default(),
        }];
        let result = inject(content, &blocks, &make_default_input_blocks("new")).unwrap();
        assert!(!result.ends_with('\n'));
    }

    #[test]
    fn test_inject_multiple_blocks() {
        let content =
            "// injm begin\nold one\n// injm end\ncode\n// injm begin\nold two\n// injm end\n";
        let blocks = vec![
            MarkerBlock {
                span: SourceSpan::new(0, 2),
                role: BlockRole::Output { id: None },
                content: "".to_string(),
                config: MarkerConfig::default(),
            },
            MarkerBlock {
                span: SourceSpan::new(4, 6),
                role: BlockRole::Output { id: None },
                content: "".to_string(),
                config: MarkerConfig::default(),
            },
        ];
        let result = inject(content, &blocks, &make_default_input_blocks("new")).unwrap();
        assert!(!result.contains("old one"));
        assert!(!result.contains("old two"));
        assert_eq!(result.matches("new").count(), 2);
    }

    #[test]
    fn test_inject_empty_block() {
        let content = "// injm begin\n// injm end\n";
        let blocks = vec![MarkerBlock {
            span: SourceSpan::new(0, 1),
            role: BlockRole::Output { id: None },
            content: "".to_string(),
            config: MarkerConfig::default(),
        }];
        let result = inject(content, &blocks, &make_default_input_blocks("new content")).unwrap();
        assert!(result.contains("new content"));
    }

    #[test]
    fn test_inject_multiline_stdin() {
        let content = "// injm begin\nold\n// injm end\n";
        let blocks = vec![MarkerBlock {
            span: SourceSpan::new(0, 2),
            role: BlockRole::Output { id: None },
            content: "".to_string(),
            config: MarkerConfig::default(),
        }];
        let result = inject(
            content,
            &blocks,
            &make_default_input_blocks("line one\nline two\nline three"),
        )
        .unwrap();
        assert!(result.contains("line one\nline two\nline three"));
    }

    #[test]
    fn test_inject_with_id() {
        let content = "\
// injm begin <first
old first
// injm end 
// injm begin <second
old second
// injm end 
";
        let blocks = vec![
            MarkerBlock {
                span: SourceSpan::new(0, 2),
                role: BlockRole::Output {
                    id: Some("first".to_string()),
                },
                content: "".to_string(),
                config: MarkerConfig::default(),
            },
            MarkerBlock {
                span: SourceSpan::new(3, 5),
                role: BlockRole::Output {
                    id: Some("second".to_string()),
                },
                content: "".to_string(),
                config: MarkerConfig::default(),
            },
        ];

        let input_blocks = vec![MarkerBlock {
            span: SourceSpan::new(0, 0),
            role: BlockRole::Input {
                ids: vec!["first".to_string()],
            },
            content: "new content".to_string(),
            config: MarkerConfig::default(),
        }];

        let result = inject(content, &blocks, &input_blocks).unwrap();
        assert!(result.contains("new content"));
        assert!(!result.contains("old first"));
        assert!(result.contains("old second"));
    }

    #[test]
    fn test_inject_when_no_ids() {
        let content = "\
// injm begin <first
old first
// injm end 
// injm begin <second
old second
// injm end 
// injm begin
// injm end
";
        let blocks = vec![
            MarkerBlock {
                span: SourceSpan::new(0, 2),
                role: BlockRole::Output {
                    id: Some("first".to_string()),
                },
                content: "".to_string(),
                config: MarkerConfig::default(),
            },
            MarkerBlock {
                span: SourceSpan::new(3, 5),
                role: BlockRole::Output {
                    id: Some("second".to_string()),
                },
                content: "".to_string(),
                config: MarkerConfig::default(),
            },
            MarkerBlock {
                span: SourceSpan::new(6, 8),
                role: BlockRole::Output { id: None },
                content: "".to_string(),
                config: MarkerConfig::default(),
            },
        ];
        let result = inject(content, &blocks, &make_default_input_blocks("new content")).unwrap();
        assert!(result.contains("old first"));
        assert!(result.contains("old second"));
        assert_eq!(result.matches("new content").count(), 1);
    }

    #[test]
    fn test_inject_multiple_ids() {
        let content = "\
// injm begin <first
old first
// injm end 
// injm begin <second
old second
// injm end 
// injm begin <third
old third
// injm end 
";
        let blocks = vec![
            MarkerBlock {
                span: SourceSpan::new(0, 2),
                role: BlockRole::Output {
                    id: Some("first".to_string()),
                },
                content: "".to_string(),
                config: MarkerConfig::default(),
            },
            MarkerBlock {
                span: SourceSpan::new(3, 5),
                role: BlockRole::Output {
                    id: Some("second".to_string()),
                },
                content: "".to_string(),
                config: MarkerConfig::default(),
            },
            MarkerBlock {
                span: SourceSpan::new(6, 8),
                role: BlockRole::Output {
                    id: Some("third".to_string()),
                },
                content: "".to_string(),
                config: MarkerConfig::default(),
            },
        ];

        let input_blocks = vec![MarkerBlock {
            span: SourceSpan::new(0, 0),
            role: BlockRole::Input {
                ids: vec!["first".to_string(), "third".to_string()],
            },
            content: "new content".to_string(),
            config: MarkerConfig::default(),
        }];

        let result = inject(content, &blocks, &input_blocks).unwrap();
        assert!(!result.contains("old first"));
        assert!(result.contains("old second"));
        assert!(!result.contains("old third"));
        assert_eq!(result.matches("new content").count(), 2);
    }
}
