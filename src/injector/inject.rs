use super::{InjectorError, Result};
use crate::types::MarkerBlock;

pub fn inject(
    content: &str,
    output_blocks: &[MarkerBlock],
    input_blocks: &[MarkerBlock],
) -> Result<String> {
    let mut lines: Vec<&str> = content.lines().collect();

    // Use reversed iteration to avoid changes of line number.
    for block in output_blocks.iter().rev() {
        if let Some(input_block) = input_blocks.iter().find(|b| b.matches_output(block)) {
            if input_block.content.is_empty() {
                return Err(InjectorError::EmptyInputContent);
            }
            lines = inject_into_a_block(&lines, block, &input_block.content);
        }
    }

    let mut result = lines.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }

    Ok(result)
}

fn inject_into_a_block<'a>(lines: &[&'a str], block: &MarkerBlock, stdin: &'a str) -> Vec<&'a str> {
    // Content to be replaced is in (begin_line, end_line).
    let before = &lines[block.span.before_lines()];
    let after = &lines[block.span.after_lines()];

    let mut injected = Vec::new();
    injected.extend_from_slice(before);
    injected.push(stdin);
    injected.extend_from_slice(after);

    injected
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
