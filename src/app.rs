use std::process::Command;

use anyhow::Result;
use ratatui::widgets::ListState;
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
};

use crate::diff_file::{DiffFile, DiffLine};

pub struct App {
    pub files: Vec<DiffFile>,
    pub selected_file: usize,
    pub file_list_state: ListState,
    pub scroll_offset: usize,
    pub show_side_by_side: bool,
    pub show_shortcuts: bool,
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}

impl App {
    pub fn new(show_side_by_side: bool) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            files: Vec::new(),
            selected_file: 0,
            file_list_state: state,
            scroll_offset: 0,
            show_side_by_side,
            show_shortcuts: true,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn load_diff(&mut self, args: &str) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("diff").arg("--no-prefix");

        if !args.is_empty() {
            for arg in args.split_whitespace() {
                cmd.arg(arg);
            }
        }

        let output = cmd.output()?;
        let diff_text = String::from_utf8_lossy(&output.stdout);

        self.files = parse_diff(&diff_text);

        if !self.files.is_empty() {
            self.file_list_state.select(Some(0));
        }

        Ok(())
    }

    pub fn next_file(&mut self) {
        if !self.files.is_empty() {
            self.selected_file = (self.selected_file + 1) % self.files.len();
            self.file_list_state.select(Some(self.selected_file));
            self.scroll_offset = 0;
        }
    }

    pub fn previous_file(&mut self) {
        if !self.files.is_empty() {
            self.selected_file = if self.selected_file == 0 {
                self.files.len() - 1
            } else {
                self.selected_file - 1
            };
            self.file_list_state.select(Some(self.selected_file));
            self.scroll_offset = 0;
        }
    }

    pub fn scroll_down(&mut self) {
        if let Some(file) = self.files.get(self.selected_file)
            && self.scroll_offset + 1 < file.line_count()
        {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn toggle_view_mode(&mut self, width: u16) {
        self.show_side_by_side = width >= 120 && !self.show_side_by_side;
    }

    pub fn toggle_shortcuts(&mut self) {
        self.show_shortcuts = !self.show_shortcuts;
    }

    pub fn get_syntax_for_file(
        &self,
        filename: &str,
    ) -> Option<&syntect::parsing::SyntaxReference> {
        // Try by extension first
        if let Some(extension) = std::path::Path::new(filename).extension()
            && let Some(ext_str) = extension.to_str()
        {
            // Handle TypeScript and JavaScript specifically
            match ext_str {
                "ts" | "tsx" => {
                    // TypeScript isn't in default syntect, use JavaScript syntax
                    return self
                        .syntax_set
                        .find_syntax_by_extension("js")
                        .or_else(|| self.syntax_set.find_syntax_by_name("JavaScript"));
                }
                "js" | "jsx" => return self.syntax_set.find_syntax_by_extension("js"),
                "rs" => return self.syntax_set.find_syntax_by_extension("rs"),
                "py" => return self.syntax_set.find_syntax_by_extension("py"),
                "go" => return self.syntax_set.find_syntax_by_extension("go"),
                "java" => return self.syntax_set.find_syntax_by_extension("java"),
                "cpp" | "cc" | "cxx" => return self.syntax_set.find_syntax_by_extension("cpp"),
                "c" => return self.syntax_set.find_syntax_by_extension("c"),
                "h" | "hpp" => return self.syntax_set.find_syntax_by_extension("h"),
                _ => {}
            }
        }

        // Fall back to the original method
        self.syntax_set
            .find_syntax_for_file(filename)
            .ok()
            .flatten()
    }

    pub fn get_theme(&self, theme_name: &str) -> &Theme {
        &self.theme_set.themes[theme_name]
    }

    pub fn get_syntax_set(&self) -> &SyntaxSet {
        &self.syntax_set
    }
}

fn parse_diff(diff_text: &str) -> Vec<DiffFile> {
    let mut files = Vec::new();
    let mut current_file: Option<DiffFile> = None;
    let mut old_line_num = 0u32;
    let mut new_line_num = 0u32;

    for line in diff_text.lines() {
        if line.starts_with("diff --git") {
            if let Some(file) = current_file.take() {
                files.push(file);
            }

            // Extract filename from "diff --git a/file b/file"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let filename = parts[3].trim_start_matches("b/");
                current_file = Some(DiffFile::new(filename));
            }
        } else if line.starts_with("@@") {
            // Parse hunk header: @@ -old_start,old_count +new_start,new_count @@
            if let Some(ref mut file) = current_file {
                if let Some((old_info, new_info)) = line.split_once(' ').and_then(|(_, rest)| {
                    rest.split_once(' ').map(|(old, new)| {
                        (old.trim_start_matches('-'), new.trim_start_matches('+'))
                    })
                }) {
                    old_line_num = old_info
                        .split(',')
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1);
                    new_line_num = new_info
                        .split(',')
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1);
                }

                file.add_line(DiffLine::new_header(line));
            }
        } else if line.starts_with('+') && !line.starts_with("+++") {
            if let Some(ref mut file) = current_file {
                file.add_line(DiffLine::new_added(line, new_line_num));
                new_line_num += 1;
            }
        } else if line.starts_with('-') && !line.starts_with("---") {
            if let Some(ref mut file) = current_file {
                file.add_line(DiffLine::new_removed(line, old_line_num));
                old_line_num += 1;
            }
        } else if line.starts_with(' ')
            && let Some(ref mut file) = current_file
        {
            file.add_line(DiffLine::new_context(line, old_line_num, new_line_num));
            old_line_num += 1;
            new_line_num += 1;
        }
    }

    if let Some(file) = current_file {
        files.push(file);
    }

    files
}
