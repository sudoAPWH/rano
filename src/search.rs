/// Search and replace functionality. Port of nano's search.c.

use crate::chars;
use crate::definitions::*;
use crate::editor::Editor;

use regex::Regex;
use std::io;

impl Editor {
    /// Initiate a search (^W).
    pub fn do_search(&mut self, direction: Direction) -> io::Result<()> {
        let prompt = match direction {
            Direction::Forward => "Search",
            Direction::Backward => "Search [Backwards]",
        };

        let needle = self.do_prompt(prompt)?;
        if needle.is_empty() {
            if self.last_search.is_empty() {
                self.statusline(MessageType::Ahem, "Cancelled");
                return Ok(());
            }
            // Use last search
        } else {
            self.last_search = needle;
        }

        self.search_flags = match direction {
            Direction::Backward => EditorFlags::BACKWARDS_SEARCH,
            Direction::Forward => EditorFlags::empty(),
        };

        self.go_looking();
        self.refresh_needed = true;
        Ok(())
    }

    /// Search forward for the next occurrence without prompting.
    pub fn do_findnext(&mut self) -> io::Result<()> {
        if self.last_search.is_empty() {
            self.statusline(MessageType::Ahem, "No current search pattern");
            return Ok(());
        }
        self.search_flags.remove(EditorFlags::BACKWARDS_SEARCH);
        self.go_looking();
        self.refresh_needed = true;
        Ok(())
    }

    /// Search backward for the next occurrence without prompting.
    pub fn do_findprevious(&mut self) -> io::Result<()> {
        if self.last_search.is_empty() {
            self.statusline(MessageType::Ahem, "No current search pattern");
            return Ok(());
        }
        self.search_flags.insert(EditorFlags::BACKWARDS_SEARCH);
        self.go_looking();
        self.refresh_needed = true;
        Ok(())
    }

    /// Perform the actual search for last_search.
    fn go_looking(&mut self) {
        let needle = self.last_search.clone();
        let case_sensitive = self.flags.contains(EditorFlags::CASE_SENSITIVE);
        let use_regexp = self.flags.contains(EditorFlags::USE_REGEXP);
        let backward = self.search_flags.contains(EditorFlags::BACKWARDS_SEARCH);

        let buf = &self.buffers[self.current_buf];
        let start_line = buf.current;
        let start_x = buf.current_x;
        let line_count = buf.lines.len();

        // Try to find the needle
        let found = if use_regexp {
            self.search_regexp(&needle, start_line, start_x, backward, line_count)
        } else {
            self.search_plain(&needle, start_line, start_x, backward, case_sensitive, line_count)
        };

        match found {
            Some((line, col)) => {
                let tabsize = self.tabsize;
                let editwinrows = self.editwinrows;
                let buf = self.current_buffer_mut();
                buf.current = line;
                buf.current_x = col;
                buf.placewewant = chars::wideness(
                    &buf.lines[line].data,
                    col,
                    tabsize,
                );
                // Center the found line
                if line < buf.edittop || line >= buf.edittop + editwinrows {
                    buf.edittop = line.saturating_sub(editwinrows / 2);
                }
            }
            None => {
                self.statusline(MessageType::Ahem, &format!("\"{}\" not found", needle));
            }
        }
    }

    /// Plain text search.
    fn search_plain(
        &self,
        needle: &str,
        start_line: usize,
        start_x: usize,
        backward: bool,
        case_sensitive: bool,
        line_count: usize,
    ) -> Option<(usize, usize)> {
        let buf = &self.buffers[self.current_buf];
        let needle_lower = if case_sensitive { needle.to_string() } else { needle.to_lowercase() };

        let mut line_idx = start_line;
        let mut first_iteration = true;
        let mut wrapped = false;

        loop {
            let line_data = &buf.lines[line_idx].data;
            let haystack = if case_sensitive {
                line_data.clone()
            } else {
                line_data.to_lowercase()
            };

            let search_from = if first_iteration && line_idx == start_line {
                if backward {
                    // Search in the portion before the cursor
                    0
                } else {
                    // Skip past the current position
                    chars::step_right(line_data, start_x).min(line_data.len())
                }
            } else {
                0
            };

            if backward {
                let end = if first_iteration && line_idx == start_line {
                    start_x
                } else {
                    haystack.len()
                };
                if let Some(pos) = haystack[..end].rfind(&needle_lower) {
                    return Some((line_idx, pos));
                }
            } else {
                if search_from <= haystack.len() {
                    if let Some(pos) = haystack[search_from..].find(&needle_lower) {
                        return Some((line_idx, search_from + pos));
                    }
                }
            }

            first_iteration = false;

            // Move to the next/previous line
            if backward {
                if line_idx == 0 {
                    if wrapped {
                        return None;
                    }
                    line_idx = line_count - 1;
                    wrapped = true;
                } else {
                    line_idx -= 1;
                }
            } else {
                line_idx += 1;
                if line_idx >= line_count {
                    if wrapped {
                        return None;
                    }
                    line_idx = 0;
                    wrapped = true;
                }
            }

            // Full circle
            if line_idx == start_line && wrapped {
                // One last check on the start line for the portion we haven't checked
                let line_data = &buf.lines[line_idx].data;
                let haystack = if case_sensitive {
                    line_data.clone()
                } else {
                    line_data.to_lowercase()
                };

                if backward {
                    if let Some(pos) = haystack[start_x..].rfind(&needle_lower) {
                        return Some((line_idx, start_x + pos));
                    }
                } else if start_x > 0 {
                    if let Some(pos) = haystack[..start_x].find(&needle_lower) {
                        return Some((line_idx, pos));
                    }
                }
                return None;
            }
        }
    }

    /// Regex search.
    fn search_regexp(
        &self,
        pattern: &str,
        start_line: usize,
        start_x: usize,
        backward: bool,
        line_count: usize,
    ) -> Option<(usize, usize)> {
        let case_insensitive = !self.flags.contains(EditorFlags::CASE_SENSITIVE);
        let re_pattern = if case_insensitive {
            format!("(?i){}", pattern)
        } else {
            pattern.to_string()
        };

        let re = match Regex::new(&re_pattern) {
            Ok(r) => r,
            Err(e) => {
                // Can't set status here since we don't have &mut self
                return None;
            }
        };

        let buf = &self.buffers[self.current_buf];
        let mut line_idx = start_line;
        let mut first_iteration = true;
        let mut wrapped = false;

        loop {
            let line_data = &buf.lines[line_idx].data;

            let search_from = if first_iteration && line_idx == start_line {
                if backward { 0 } else {
                    chars::step_right(line_data, start_x).min(line_data.len())
                }
            } else {
                0
            };

            if backward {
                let end = if first_iteration && line_idx == start_line {
                    start_x
                } else {
                    line_data.len()
                };
                // Find the last match before `end`
                let mut last_match = None;
                for m in re.find_iter(&line_data[..end]) {
                    last_match = Some(m.start());
                }
                if let Some(pos) = last_match {
                    return Some((line_idx, pos));
                }
            } else if search_from <= line_data.len() {
                if let Some(m) = re.find(&line_data[search_from..]) {
                    return Some((line_idx, search_from + m.start()));
                }
            }

            first_iteration = false;

            if backward {
                if line_idx == 0 {
                    if wrapped { return None; }
                    line_idx = line_count - 1;
                    wrapped = true;
                } else {
                    line_idx -= 1;
                }
            } else {
                line_idx += 1;
                if line_idx >= line_count {
                    if wrapped { return None; }
                    line_idx = 0;
                    wrapped = true;
                }
            }

            if line_idx == start_line && wrapped {
                return None;
            }
        }
    }

    /// Search and replace (^\).
    pub fn do_replace(&mut self) -> io::Result<()> {
        let needle = self.do_prompt("Search (to replace)")?;
        if needle.is_empty() {
            self.statusline(MessageType::Info, "Cancelled");
            return Ok(());
        }
        self.last_search = needle.clone();

        let replacement = self.do_prompt("Replace with")?;

        let case_sensitive = self.flags.contains(EditorFlags::CASE_SENSITIVE);
        let use_regexp = self.flags.contains(EditorFlags::USE_REGEXP);
        let mut count = 0;

        let buf = self.current_buffer_mut();
        let line_count = buf.lines.len();

        if use_regexp {
            let case_flag = if !case_sensitive { "(?i)" } else { "" };
            if let Ok(re) = Regex::new(&format!("{}{}", case_flag, needle)) {
                for i in 0..line_count {
                    let new_data = re.replace_all(&buf.lines[i].data, replacement.as_str()).to_string();
                    if new_data != buf.lines[i].data {
                        count += 1;
                        buf.lines[i].data = new_data;
                    }
                }
            }
        } else {
            for i in 0..line_count {
                let line = &buf.lines[i].data;
                let new_data = if case_sensitive {
                    line.replace(&needle, &replacement)
                } else {
                    // Case-insensitive replace
                    let lower_line = line.to_lowercase();
                    let lower_needle = needle.to_lowercase();
                    let mut result = String::new();
                    let mut last_end = 0;
                    for (start, _) in lower_line.match_indices(&lower_needle) {
                        result.push_str(&line[last_end..start]);
                        result.push_str(&replacement);
                        last_end = start + needle.len();
                    }
                    result.push_str(&line[last_end..]);
                    result
                };
                if new_data != buf.lines[i].data {
                    count += 1;
                    buf.lines[i].data = new_data;
                }
            }
        }

        if count > 0 {
            self.set_modified();
            self.statusline(
                MessageType::Info,
                &format!("Replaced in {} line{}", count, if count == 1 { "" } else { "s" }),
            );
        } else {
            self.statusline(MessageType::Ahem, &format!("\"{}\" not found", needle));
        }

        self.refresh_needed = true;
        Ok(())
    }

    /// Go to a specific line and column (^_).
    pub fn do_gotolinecolumn(&mut self) -> io::Result<()> {
        let input = self.do_prompt("Enter line number, column number")?;
        if input.is_empty() {
            self.statusline(MessageType::Info, "Cancelled");
            return Ok(());
        }

        let (line_opt, col_opt) = crate::utils::parse_line_column(&input);
        let target_line = line_opt.unwrap_or(1).max(1) as usize - 1;
        let target_col = col_opt.unwrap_or(1).max(1) as usize - 1;

        let tabsize = self.tabsize;
        let editwinrows = self.editwinrows;
        let buf = self.current_buffer_mut();
        let max_line = buf.lines.len() - 1;
        buf.current = target_line.min(max_line);
        buf.current_x = chars::actual_x(&buf.lines[buf.current].data, target_col, tabsize);
        buf.placewewant = chars::wideness(
            &buf.lines[buf.current].data,
            buf.current_x,
            tabsize,
        );

        // Center the target line
        buf.edittop = buf.current.saturating_sub(editwinrows / 2);

        self.refresh_needed = true;
        Ok(())
    }
}
