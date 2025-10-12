#[derive(Debug, Clone)]
pub struct DiffFile {
    name: String,
    status: char, // M, A, D, etc.
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: LineType,
    pub old_line_num: Option<u32>,
    pub new_line_num: Option<u32>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineType {
    Context,
    Added,
    Removed,
    Header,
}

impl DiffFile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: 'M', // Default to modified
            lines: Vec::new(),
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn add_line(&mut self, line: DiffLine) {
        self.lines.push(line);
    }

    pub fn get_status(&self) -> char {
        self.status
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl DiffLine {
    pub fn new_header(line: &str) -> Self {
        Self {
            line_type: LineType::Header,
            old_line_num: None,
            new_line_num: None,
            content: line.to_string(),
        }
    }

    pub fn new_added(line: &str, line_number: u32) -> Self {
        Self {
            line_type: LineType::Added,
            old_line_num: None,
            new_line_num: Some(line_number),
            content: line[1..].to_string(),
        }
    }

    pub fn new_removed(line: &str, line_number: u32) -> Self {
        Self {
            line_type: LineType::Removed,
            old_line_num: Some(line_number),
            new_line_num: None,
            content: line[1..].to_string(),
        }
    }

    pub fn new_context(line: &str, old_line_num: u32, new_line_num: u32) -> Self {
        Self {
            line_type: LineType::Context,
            old_line_num: Some(old_line_num),
            new_line_num: Some(new_line_num),
            content: line[1..].to_string(),
        }
    }
}
