use crate::parser::ParserError;

use super::Result;
use tree_sitter_language_pack::{Node, ProcessConfig, get_parser, process};

#[derive(Debug)]
pub(super) struct Comment {
    pub text: String,
    pub start_line: usize,
    pub end_line: usize,
}

pub(super) fn extract_comments(content: &str, lang: &str) -> Result<Vec<Comment>> {
    if lang == "markdown" {
        let comments = extract_markdown_comments(content)?;
        return Ok(comments);
    }

    let mut comments: Vec<Comment> = Vec::new();

    // Query all comments.
    let mut config = ProcessConfig::new(lang);
    config.comments = true;

    let result = process(content, &config)?;
    for comment in result.comments {
        comments.push(Comment {
            text: comment.text.trim().to_string(),
            start_line: comment.span.start_line,
            end_line: comment.span.end_line,
        });
    }

    Ok(comments)
}

fn extract_markdown_comments(content: &str) -> Result<Vec<Comment>> {
    let mut parser = get_parser("markdown")?;
    let tree = parser.parse(content).ok_or(ParserError::ParseFailed {
        content: content.to_string(),
    })?;
    let mut comments = Vec::new();
    let lines: Vec<&str> = content.split('\n').collect();
    collect_markdown_comments(tree.root_node(), &lines, &mut comments);

    Ok(comments)
}

fn collect_markdown_comments(node: Node, lines: &[&str], comments: &mut Vec<Comment>) {
    let kind = node.kind();

    if kind == "html_block" {
        let end_position = node.end_position();
        let start_line = node.start_position().row;

        let end_line = if end_position.column == 0 && end_position.row > start_line {
            end_position.row - 1
        } else {
            end_position.row
        };

        let raw_text = lines[start_line..=end_line].join("\n");
        extract_comments_from_block(&raw_text, start_line, comments);

        return;
    }

    for index in 0..node.child_count() {
        if let Some(child) = node.child(index as u32) {
            collect_markdown_comments(child, lines, comments);
        }
    }
}

fn extract_comments_from_block(
    raw_text: &str,
    block_start_line: usize,
    comments: &mut Vec<Comment>,
) {
    const OPEN: &str = "<!--";
    const CLOSE: &str = "-->";

    let mut search_from = 0;

    while search_from < raw_text.len() {
        let Some(relative_start) = raw_text[search_from..].find(OPEN) else {
            break;
        };

        let comment_start = search_from + relative_start;
        let text_start = comment_start + OPEN.len();
        let Some(relative_end) = raw_text[text_start..].find(CLOSE) else {
            break;
        };

        let text_end = text_start + relative_end;
        let comment_end = text_end + CLOSE.len();

        let text = raw_text[text_start..text_end].trim().to_string();

        let start_line = block_start_line + count_newlines(&raw_text[..comment_start]);
        let end_line = block_start_line + count_newlines(&raw_text[..comment_end]);

        if !text.is_empty() {
            comments.push(Comment {
                text,
                start_line,
                end_line,
            });
        }

        search_from = comment_end;
    }
}

fn count_newlines(text: &str) -> usize {
    text.as_bytes()
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_comments() {
        let content = r"
fn main() {
    // hello world
    let x = 1;
}
";
        let comments = extract_comments(content, "rust").unwrap();
        assert!(comments.iter().any(|c| c.text.contains("hello world")));
    }

    #[test]
    fn test_extract_multiple_comments() {
        let content = r"
// first comment
fn main() {
    // second comment
}
";
        let comments = extract_comments(content, "rust").unwrap();
        assert_eq!(comments.len(), 2);
    }

    #[test]
    fn test_comment_line_numbers() {
        let content = r"fn main() {
    // hello
}";
        let comments = extract_comments(content, "rust").unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].start_line, 1);
        assert_eq!(comments[0].end_line, 1);
    }

    #[test]
    fn test_markdown_comments_extracted() {
        let content = r"
# Title

<!-- this is an html comment -->
";
        // tree-sitter-markdown doesn't treat HTML comments as comment nodes
        let comments = extract_comments(content, "markdown").unwrap();
        assert_eq!(comments.len(), 1);
        assert!(
            comments
                .iter()
                .any(|c| c.text.contains("this is an html comment"))
        );
        assert!(
            comments
                .iter()
                .all(|c| !c.text.contains("<!--") && !c.text.contains("-->"))
        );
    }

    #[test]
    fn test_mutiple_line_markdown_comments_extracted() {
        let content = r"
# Title

<!-- 
this is a
multiple line
markdown comment
-->
";
        let comments = extract_comments(content, "markdown").unwrap();
        assert_eq!(comments.len(), 1);
        assert!(comments.iter().any(|c| {
            c.text
                .contains("this is a\nmultiple line\nmarkdown comment")
        }));
        assert!(
            comments
                .iter()
                .all(|c| !c.text.contains("<!--") && !c.text.contains("-->"))
        );
    }

    #[test]
    fn test_mutiple_line_markdown_comments_with_codeblock_extracted() {
        let content = r"
# Title

<!-- 
this is a
multiple line
markdown comment
-->
```
<!-- inside codeblock -->
<!-- multiple
lines inside
codeblock-->
```
";
        // tree-sitter-markdown doesn't treat HTML comments as comment nodes
        let comments = extract_comments(content, "markdown").unwrap();
        assert_eq!(comments.len(), 1);
        assert!(comments.iter().any(|c| {
            c.text
                .contains("this is a\nmultiple line\nmarkdown comment")
        }));
        assert!(comments.iter().all(|c| {
            !c.text.contains("inside codeblock")
                && !c.text.contains("multiple\nlines inside\ncodeblock")
        }));
        assert!(
            comments
                .iter()
                .all(|c| !c.text.contains("<!--") && !c.text.contains("-->"))
        );
    }

    #[test]
    fn test_no_comments() {
        let content = r"
fn main() {
    let x = 1;
}
";
        let comments = extract_comments(content, "rust").unwrap();
        assert_eq!(comments.len(), 0);
    }

    #[test]
    fn test_extract_go_comments() {
        let content = r"
// first comment
// second comment
// third comment
func main() {
}
";
        let comments = extract_comments(content, "go").unwrap();
        assert_eq!(comments.len(), 3);
    }

    #[test]
    fn test_extract_block_comments() {
        let content = r"
/*
 This is a
 multiple
 line comment
 (aka block comment)
 */
int main(void) {}
";
        let comments = extract_comments(content, "c").unwrap();
        assert_eq!(comments.len(), 1);
    }

    #[test]
    fn test_extract_python_comments() {
        let content = r"
# hello world
def main():
    pass
";
        let comments = extract_comments(content, "python").unwrap();
        assert!(comments.iter().any(|c| c.text.contains("hello world")));
    }

    #[test]
    fn test_extract_latex_comments() {
        let content = r"
\section{injm}

% hello tex
";
        let comments = extract_comments(content, "latex").unwrap();
        assert!(comments.iter().any(|c| c.text.contains("hello tex")));
    }

    #[test]
    fn test_string_is_not_comment() {
        let content = r#"
fn main() {
    let x = "// this is not a comment";
}
"#;
        let comments = extract_comments(content, "rust").unwrap();
        assert_eq!(comments.len(), 0);
    }
}
