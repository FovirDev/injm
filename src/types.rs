use std::path::PathBuf;

#[derive(Debug)]
pub struct MarkerBlock {
    pub span: SourceSpan,
    pub role: BlockRole,
    pub content: String,
    pub config: MarkerConfig,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct MarkerConfig {
    pub offset: usize,
    pub trim: bool,
    pub indent: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub begin_marker: usize,
    pub end_marker: usize,
}

#[derive(Debug)]
pub enum BlockRole {
    Output { id: Option<String> },
    Input { ids: Vec<String> },
}

#[derive(Debug)]
pub struct ParsedFile {
    pub content: String,
    pub blocks: Vec<MarkerBlock>,
    pub path: PathBuf,
}

impl SourceSpan {
    pub fn new(begin_marker: usize, end_marker: usize) -> Self {
        Self {
            begin_marker,
            end_marker,
        }
    }

    pub fn content_lines(&self) -> std::ops::Range<usize> {
        self.begin_marker + 1..self.end_marker
    }

    pub fn display_lines(&self) -> String {
        format!("{}-{}", self.begin_marker + 1, self.end_marker + 1)
    }

    pub fn before_lines(&self, offset: usize) -> std::ops::RangeToInclusive<usize> {
        ..=self.begin_marker.saturating_add(offset)
    }

    pub fn after_lines(&self, offset: usize) -> std::ops::RangeFrom<usize> {
        self.end_marker.saturating_sub(offset)..
    }
}
