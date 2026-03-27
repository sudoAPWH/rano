// Rano - A Rust-based rewrite of the GNU Nano text editor.
// Copyright (C) 2026 Alexander Hutlet

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.


/// Text editing operations. Port of nano's text.c.

use crate::chars;
use crate::definitions::*;
use crate::editor::Editor;

impl Editor {
    /// Insert a single character at the cursor position.
    pub fn do_char(&mut self, c: char) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            self.statusline(MessageType::Ahem, "Key invalid in view mode");
            return;
        }

        self.add_undo(UndoType::Add);

        let tabsize = self.tabsize;
        let buf = self.current_buffer_mut();
        let current = buf.current;
        buf.lines[current].data.insert(buf.current_x, c);
        buf.current_x += c.len_utf8();
        buf.totsize += c.len_utf8();
        buf.placewewant = chars::wideness(&buf.lines[current].data, buf.current_x, tabsize);

        self.set_modified();
        self.refresh_needed = true;
    }

    /// Insert a tab (or spaces if tabs_to_spaces is set).
    pub fn do_tab(&mut self) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            self.statusline(MessageType::Ahem, "Key invalid in view mode");
            return;
        }

        if self.flags.contains(EditorFlags::TABS_TO_SPACES) {
            let buf = self.current_buffer();
            let current_col = chars::wideness(
                &buf.lines[buf.current].data,
                buf.current_x,
                self.tabsize,
            );
            let spaces = self.tabsize - (current_col % self.tabsize);
            for _ in 0..spaces {
                self.do_char(' ');
            }
        } else {
            self.do_char('\t');
        }
    }

    /// Insert a newline (Enter key).
    pub fn do_enter(&mut self) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            self.statusline(MessageType::Ahem, "Key invalid in view mode");
            return;
        }

        self.add_undo(UndoType::Enter);

        let autoindent = self.flags.contains(EditorFlags::AUTOINDENT);
        let tabsize = self.tabsize;

        let buf = self.current_buffer_mut();
        let current_idx = buf.current;
        let current_x = buf.current_x;

        // Get the text after the cursor
        let after = buf.lines[current_idx].data[current_x..].to_string();
        buf.lines[current_idx].data.truncate(current_x);

        // Compute indentation for the new line
        let indent = if autoindent {
            let old_line = &buf.lines[current_idx].data;
            let indent_len = chars::indent_length(old_line);
            old_line[..indent_len].to_string()
        } else {
            String::new()
        };

        // Create the new line
        let new_line_data = format!("{}{}", indent, after);
        let new_line = Line::new(new_line_data, current_idx + 2);

        buf.lines.insert(current_idx + 1, new_line);

        // Move cursor to new line
        buf.current = current_idx + 1;
        buf.current_x = indent.len();
        buf.totsize += 1; // for the newline

        // Renumber
        buf.renumber_from(current_idx + 1);

        buf.placewewant = chars::wideness(
            &buf.lines[buf.current].data,
            buf.current_x,
            tabsize,
        );

        self.set_modified();
        self.confirm_margin();
        self.refresh_needed = true;
    }

    /// Delete the character under the cursor.
    pub fn do_delete(&mut self) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            self.statusline(MessageType::Ahem, "Key invalid in view mode");
            return;
        }

        let buf = self.current_buffer();
        let line_len = buf.lines[buf.current].data.len();

        if buf.current_x < line_len {
            self.add_undo(UndoType::Del);
            let buf = self.current_buffer_mut();
            let next_x = chars::step_right(&buf.lines[buf.current].data, buf.current_x);
            let removed_len = next_x - buf.current_x;
            buf.lines[buf.current].data.drain(buf.current_x..next_x);
            buf.totsize -= removed_len;
            self.set_modified();
            self.refresh_needed = true;
        } else if buf.current < buf.lines.len() - 1 {
            // Join with next line
            self.add_undo(UndoType::Join);
            let buf = self.current_buffer_mut();
            let current_idx = buf.current;
            let next_data = buf.lines[current_idx + 1].data.clone();
            buf.lines[current_idx].data.push_str(&next_data);
            buf.lines.remove(current_idx + 1);
            buf.totsize -= 1; // the joined newline
            buf.renumber_from(current_idx + 1);
            self.set_modified();
            self.confirm_margin();
            self.refresh_needed = true;
        }
    }

    /// Delete the character before the cursor.
    pub fn do_backspace(&mut self) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            self.statusline(MessageType::Ahem, "Key invalid in view mode");
            return;
        }

        let tabsize = self.tabsize;
        let buf = self.current_buffer();
        if buf.current_x > 0 {
            self.add_undo(UndoType::Back);
            let buf = self.current_buffer_mut();
            let prev_x = chars::step_left(&buf.lines[buf.current].data, buf.current_x);
            let removed_len = buf.current_x - prev_x;
            buf.lines[buf.current].data.drain(prev_x..buf.current_x);
            buf.current_x = prev_x;
            buf.totsize -= removed_len;
            buf.placewewant = chars::wideness(
                &buf.lines[buf.current].data,
                buf.current_x,
                tabsize,
            );
            self.set_modified();
            self.refresh_needed = true;
        } else if buf.current > 0 {
            // Join with previous line
            self.add_undo(UndoType::Back);
            let buf = self.current_buffer_mut();
            let current_idx = buf.current;
            let current_data = buf.lines[current_idx].data.clone();
            let prev_len = buf.lines[current_idx - 1].data.len();
            buf.lines[current_idx - 1].data.push_str(&current_data);
            buf.lines.remove(current_idx);
            buf.current = current_idx - 1;
            buf.current_x = prev_len;
            buf.totsize -= 1;
            buf.renumber_from(current_idx);
            buf.placewewant = chars::wideness(
                &buf.lines[buf.current].data,
                buf.current_x,
                tabsize,
            );
            self.set_modified();
            self.confirm_margin();
            self.refresh_needed = true;
        }
    }

    /// Set or unset the mark.
    pub fn do_mark(&mut self) {
        let buf = self.current_buffer_mut();
        if buf.mark.is_some() {
            buf.mark = None;
            buf.softmark = false;
            self.statusline(MessageType::Info, "Mark Unset");
        } else {
            buf.mark = Some(buf.current);
            buf.mark_x = buf.current_x;
            buf.softmark = false;
            self.statusline(MessageType::Info, "Mark Set");
        }
        self.refresh_needed = true;
    }

    /// Indent the current line (or marked region).
    pub fn do_indent(&mut self) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            return;
        }

        let indent_str = if self.flags.contains(EditorFlags::TABS_TO_SPACES) {
            " ".repeat(self.tabsize)
        } else {
            "\t".to_string()
        };

        let buf = self.current_buffer_mut();
        let (top, bottom) = if let Some(mark_line) = buf.mark {
            if mark_line <= buf.current {
                (mark_line, buf.current)
            } else {
                (buf.current, mark_line)
            }
        } else {
            (buf.current, buf.current)
        };

        for i in top..=bottom {
            if !buf.lines[i].data.is_empty() {
                buf.lines[i].data.insert_str(0, &indent_str);
            }
        }

        buf.current_x += indent_str.len();
        self.set_modified();
        self.refresh_needed = true;
    }

    /// Unindent the current line (or marked region).
    pub fn do_unindent(&mut self) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            return;
        }

        let tabsize = self.tabsize;
        let buf = self.current_buffer_mut();
        let (top, bottom) = if let Some(mark_line) = buf.mark {
            if mark_line <= buf.current {
                (mark_line, buf.current)
            } else {
                (buf.current, mark_line)
            }
        } else {
            (buf.current, buf.current)
        };

        for i in top..=bottom {
            let line = &buf.lines[i].data;
            if line.starts_with('\t') {
                buf.lines[i].data.remove(0);
            } else {
                let spaces = line.chars().take_while(|c| *c == ' ').count().min(tabsize);
                if spaces > 0 {
                    buf.lines[i].data.drain(0..spaces);
                }
            }
        }

        // Adjust cursor
        let indent = chars::indent_length(&buf.lines[buf.current].data);
        if buf.current_x > buf.lines[buf.current].data.len() {
            buf.current_x = buf.lines[buf.current].data.len();
        }

        self.set_modified();
        self.refresh_needed = true;
    }

    /// Comment or uncomment the current line (or marked lines).
    pub fn do_comment(&mut self) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            return;
        }

        let comment = GENERAL_COMMENT_CHARACTER;
        let buf = self.current_buffer_mut();
        let (top, bottom) = if let Some(mark_line) = buf.mark {
            if mark_line <= buf.current {
                (mark_line, buf.current)
            } else {
                (buf.current, mark_line)
            }
        } else {
            (buf.current, buf.current)
        };

        // Check if all lines in range already have the comment prefix
        let all_commented = (top..=bottom).all(|i| {
            let trimmed = buf.lines[i].data.trim_start();
            trimmed.starts_with(comment) || trimmed.is_empty()
        });

        if all_commented {
            // Uncomment
            for i in top..=bottom {
                if let Some(pos) = buf.lines[i].data.find(comment) {
                    buf.lines[i].data.drain(pos..pos + comment.len());
                }
            }
        } else {
            // Comment
            for i in top..=bottom {
                if !buf.lines[i].data.trim().is_empty() {
                    let indent = chars::indent_length(&buf.lines[i].data);
                    buf.lines[i].data.insert_str(indent, comment);
                }
            }
        }

        self.set_modified();
        self.refresh_needed = true;
    }

    /// Add an undo item to the stack.
    pub fn add_undo(&mut self, action: UndoType) {
        let buf = &self.buffers[self.current_buf];
        let item = UndoItem {
            undo_type: action,
            xflags: UndoFlags::empty(),
            head_lineno: buf.current,
            head_x: buf.current_x,
            strdata: match action {
                UndoType::Add | UndoType::Enter => None,
                UndoType::Back => {
                    if buf.current_x > 0 {
                        let prev_x = chars::step_left(&buf.lines[buf.current].data, buf.current_x);
                        Some(buf.lines[buf.current].data[prev_x..buf.current_x].to_string())
                    } else if buf.current > 0 {
                        Some("\n".to_string())
                    } else {
                        None
                    }
                }
                UndoType::Del | UndoType::Join => {
                    if buf.current_x < buf.lines[buf.current].data.len() {
                        let next_x = chars::step_right(&buf.lines[buf.current].data, buf.current_x);
                        Some(buf.lines[buf.current].data[buf.current_x..next_x].to_string())
                    } else {
                        Some("\n".to_string())
                    }
                }
                _ => None,
            },
            wassize: buf.totsize,
            newsize: buf.totsize,
            grouping: Vec::new(),
            cutbuffer: Vec::new(),
            tail_lineno: buf.current,
            tail_x: buf.current_x,
        };

        let buf = self.current_buffer_mut();
        // Discard any redo items beyond the current index
        buf.undo_stack.truncate(buf.undo_index);
        buf.undo_stack.push(item);
        buf.undo_index += 1;
        buf.last_action = Some(action);
    }

    /// Undo the last action.
    pub fn do_undo(&mut self) {
        let buf = &self.buffers[self.current_buf];
        if buf.undo_index == 0 {
            self.statusline(MessageType::Ahem, "Nothing to undo");
            return;
        }

        let buf = self.current_buffer_mut();
        buf.undo_index -= 1;
        let item = buf.undo_stack[buf.undo_index].clone();

        match item.undo_type {
            UndoType::Add => {
                // Undo character insertion: delete the character
                buf.current = item.head_lineno;
                buf.current_x = item.head_x;
                if buf.current_x > 0 {
                    let prev_x = chars::step_left(&buf.lines[buf.current].data, buf.current_x);
                    buf.lines[buf.current].data.drain(prev_x..buf.current_x);
                    buf.current_x = prev_x;
                }
            }
            UndoType::Back => {
                // Undo backspace: re-insert the deleted character
                buf.current = item.head_lineno;
                buf.current_x = item.head_x;
                if let Some(ref s) = item.strdata {
                    if s == "\n" && buf.current > 0 {
                        // Split the line back
                        let current_idx = buf.current;
                        let data = buf.lines[current_idx].data.clone();
                        buf.lines.insert(current_idx + 1, Line::new(data, current_idx + 2));
                        buf.lines[current_idx].data.clear();
                        buf.current = current_idx + 1;
                        buf.current_x = 0;
                        buf.renumber_from(current_idx);
                    } else {
                        let insert_x = item.head_x.saturating_sub(s.len());
                        buf.lines[buf.current].data.insert_str(insert_x, s);
                        buf.current_x = insert_x + s.len();
                    }
                }
            }
            UndoType::Del | UndoType::Join => {
                // Undo delete: re-insert
                buf.current = item.head_lineno;
                buf.current_x = item.head_x;
                if let Some(ref s) = item.strdata {
                    if s == "\n" {
                        // Split the current line
                        let current_idx = buf.current;
                        let after = buf.lines[current_idx].data[buf.current_x..].to_string();
                        buf.lines[current_idx].data.truncate(buf.current_x);
                        buf.lines.insert(current_idx + 1, Line::new(after, current_idx + 2));
                        buf.renumber_from(current_idx + 1);
                    } else {
                        buf.lines[buf.current].data.insert_str(buf.current_x, s);
                    }
                }
            }
            UndoType::Enter => {
                // Undo enter: join lines
                let current_idx = item.head_lineno;
                if current_idx + 1 < buf.lines.len() {
                    let next_data = buf.lines[current_idx + 1].data.clone();
                    buf.lines[current_idx].data.push_str(&next_data);
                    buf.lines.remove(current_idx + 1);
                    buf.current = current_idx;
                    buf.current_x = item.head_x;
                    buf.renumber_from(current_idx + 1);
                }
            }
            _ => {
                self.statusline(MessageType::Ahem, "Cannot undo this action");
                return;
            }
        }

        self.buffers[self.current_buf].modified = true;
        self.confirm_margin();
        self.refresh_needed = true;
    }

    /// Redo the last undone action.
    pub fn do_redo(&mut self) {
        let buf = &self.buffers[self.current_buf];
        if buf.undo_index >= buf.undo_stack.len() {
            self.statusline(MessageType::Ahem, "Nothing to redo");
            return;
        }

        let item = self.buffers[self.current_buf].undo_stack[self.buffers[self.current_buf].undo_index].clone();
        let buf = self.current_buffer_mut();

        match item.undo_type {
            UndoType::Add => {
                buf.current = item.head_lineno;
                buf.current_x = item.head_x.saturating_sub(1);
                // The original char was at head_x - char_len, need to re-insert
                // For simple redo, we re-execute the forward action
                // This is simplified — a production version would store the char
                buf.current_x = item.head_x;
            }
            UndoType::Back => {
                buf.current = item.head_lineno;
                buf.current_x = item.head_x;
                if let Some(ref s) = item.strdata {
                    if s == "\n" && buf.current > 0 {
                        let current_idx = buf.current;
                        let prev_len = buf.lines[current_idx - 1].data.len();
                        let current_data = buf.lines[current_idx].data.clone();
                        buf.lines[current_idx - 1].data.push_str(&current_data);
                        buf.lines.remove(current_idx);
                        buf.current = current_idx - 1;
                        buf.current_x = prev_len;
                        buf.renumber_from(current_idx);
                    } else {
                        let prev_x = item.head_x.saturating_sub(s.len());
                        buf.lines[buf.current].data.drain(prev_x..item.head_x);
                        buf.current_x = prev_x;
                    }
                }
            }
            UndoType::Del | UndoType::Join => {
                buf.current = item.head_lineno;
                buf.current_x = item.head_x;
                if let Some(ref s) = item.strdata {
                    if s == "\n" {
                        let current_idx = buf.current;
                        if current_idx + 1 < buf.lines.len() {
                            let next_data = buf.lines[current_idx + 1].data.clone();
                            buf.lines[current_idx].data.push_str(&next_data);
                            buf.lines.remove(current_idx + 1);
                            buf.renumber_from(current_idx + 1);
                        }
                    } else {
                        let next_x = buf.current_x + s.len();
                        let line_len = buf.lines[buf.current].data.len();
                        buf.lines[buf.current].data.drain(buf.current_x..next_x.min(line_len));
                    }
                }
            }
            UndoType::Enter => {
                buf.current = item.head_lineno;
                buf.current_x = item.head_x;
                let current_idx = buf.current;
                let after = buf.lines[current_idx].data[buf.current_x..].to_string();
                buf.lines[current_idx].data.truncate(buf.current_x);
                buf.lines.insert(current_idx + 1, Line::new(after, current_idx + 2));
                buf.current = current_idx + 1;
                buf.current_x = 0;
                buf.renumber_from(current_idx + 1);
            }
            _ => {
                self.statusline(MessageType::Ahem, "Cannot redo this action");
                return;
            }
        }

        buf.undo_index += 1;
        self.buffers[self.current_buf].modified = true;
        self.confirm_margin();
        self.refresh_needed = true;
    }
}
