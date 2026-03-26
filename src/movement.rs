/// Rano - A Rust-based rewrite of the GNU Nano text editor.
/// Copyright (C) 2026 Alexander Hutlet

/// This program is free software: you can redistribute it and/or modify
/// it under the terms of the GNU General Public License as published by
/// the Free Software Foundation, either version 3 of the License, or
/// (at your option) any later version.

/// This program is distributed in the hope that it will be useful,
/// but WITHOUT ANY WARRANTY; without even the implied warranty of
/// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
/// GNU General Public License for more details.

/// You should have received a copy of the GNU General Public License
/// along with this program.  If not, see <https://www.gnu.org/licenses/>.


/// Cursor movement functions. Port of nano's move.c.

use crate::chars;
use crate::editor::Editor;

impl Editor {
    /// Move to the first line of the file.
    pub fn to_first_line(&mut self) {
        let buf = &mut self.buffers[self.current_buf];
        buf.current = 0;
        buf.current_x = 0;
        buf.placewewant = 0;
        self.refresh_needed = true;
    }

    /// Move to the last line of the file.
    pub fn to_last_line(&mut self) {
        let tabsize = self.tabsize;
        let editwinrows = self.editwinrows;
        let buf = &mut self.buffers[self.current_buf];
        let last = buf.lines.len() - 1;
        buf.current = last;
        buf.current_x = buf.lines[last].data.len();
        buf.placewewant = chars::wideness(&buf.lines[last].data, buf.current_x, tabsize);
        buf.cursor_row = editwinrows as isize - 1;
        self.refresh_needed = true;
        self.focusing = false;
    }

    /// Move up one line.
    pub fn do_up(&mut self) {
        if self.buffers[self.current_buf].current == 0 {
            return;
        }

        let tabsize = self.tabsize;
        let target = self.buffers[self.current_buf].placewewant;
        let buf = &mut self.buffers[self.current_buf];
        buf.current -= 1;
        buf.current_x = chars::actual_x(&buf.lines[buf.current].data, target, tabsize);
        buf.placewewant = target;

        if buf.cursor_row > 0 {
            buf.cursor_row -= 1;
        } else {
            self.refresh_needed = true;
        }
    }

    /// Move down one line.
    pub fn do_down(&mut self) {
        let line_count = self.buffers[self.current_buf].lines.len();
        if self.buffers[self.current_buf].current >= line_count - 1 {
            return;
        }

        let tabsize = self.tabsize;
        let target = self.buffers[self.current_buf].placewewant;
        let buf = &mut self.buffers[self.current_buf];
        buf.current += 1;
        buf.current_x = chars::actual_x(&buf.lines[buf.current].data, target, tabsize);
        buf.placewewant = target;

        if buf.cursor_row < self.editwinrows as isize - 1 {
            buf.cursor_row += 1;
        } else {
            self.refresh_needed = true;
        }
    }

    /// Move left one character.
    pub fn do_left(&mut self) {
        let tabsize = self.tabsize;
        let buf = &mut self.buffers[self.current_buf];
        if buf.current_x > 0 {
            buf.current_x = chars::step_left(&buf.lines[buf.current].data, buf.current_x);
            // Skip zero-width characters
            while buf.current_x > 0 {
                if let Some(c) = buf.lines[buf.current].data[buf.current_x..].chars().next() {
                    if chars::is_zerowidth(c) {
                        buf.current_x = chars::step_left(&buf.lines[buf.current].data, buf.current_x);
                        continue;
                    }
                }
                break;
            }
        } else if buf.current > 0 {
            buf.current -= 1;
            buf.current_x = buf.lines[buf.current].data.len();
        }
        buf.placewewant = chars::wideness(
            &buf.lines[buf.current].data,
            buf.current_x,
            tabsize,
        );
        self.refresh_needed = true;
    }

    /// Move right one character.
    pub fn do_right(&mut self) {
        let tabsize = self.tabsize;
        let buf = &mut self.buffers[self.current_buf];
        let line_len = buf.lines[buf.current].data.len();
        if buf.current_x < line_len {
            buf.current_x = chars::step_right(&buf.lines[buf.current].data, buf.current_x);
            // Skip zero-width characters
            while buf.current_x < buf.lines[buf.current].data.len() {
                if let Some(c) = buf.lines[buf.current].data[buf.current_x..].chars().next() {
                    if chars::is_zerowidth(c) {
                        buf.current_x = chars::step_right(&buf.lines[buf.current].data, buf.current_x);
                        continue;
                    }
                }
                break;
            }
        } else if buf.current < buf.lines.len() - 1 {
            buf.current += 1;
            buf.current_x = 0;
        }
        buf.placewewant = chars::wideness(
            &buf.lines[buf.current].data,
            buf.current_x,
            tabsize,
        );
        self.refresh_needed = true;
    }

    /// Move to the beginning of the current line.
    pub fn do_home(&mut self) {
        let tabsize = self.tabsize;
        let buf = &mut self.buffers[self.current_buf];
        let indent = chars::indent_length(&buf.lines[buf.current].data);
        if buf.current_x == indent || indent >= buf.lines[buf.current].data.len() {
            buf.current_x = 0;
        } else {
            buf.current_x = indent;
        }
        buf.placewewant = chars::wideness(
            &buf.lines[buf.current].data,
            buf.current_x,
            tabsize,
        );
        self.refresh_needed = true;
    }

    /// Move to the end of the current line.
    pub fn do_end(&mut self) {
        let tabsize = self.tabsize;
        let buf = &mut self.buffers[self.current_buf];
        buf.current_x = buf.lines[buf.current].data.len();
        buf.placewewant = chars::wideness(
            &buf.lines[buf.current].data,
            buf.current_x,
            tabsize,
        );
        self.refresh_needed = true;
    }

    /// Move up almost one screenful.
    pub fn do_page_up(&mut self) {
        let tabsize = self.tabsize;
        let mustmove = if self.editwinrows < 3 { 1 } else { self.editwinrows - 2 };
        let buf = &mut self.buffers[self.current_buf];

        if buf.current < mustmove {
            buf.current = 0;
            buf.current_x = 0;
            buf.placewewant = 0;
        } else {
            let target = buf.placewewant;
            buf.current -= mustmove;
            buf.current_x = chars::actual_x(&buf.lines[buf.current].data, target, tabsize);
        }

        self.refresh_needed = true;
    }

    /// Move down almost one screenful.
    pub fn do_page_down(&mut self) {
        let tabsize = self.tabsize;
        let mustmove = if self.editwinrows < 3 { 1 } else { self.editwinrows - 2 };
        let line_count = self.buffers[self.current_buf].lines.len();
        let buf = &mut self.buffers[self.current_buf];

        if buf.current + mustmove >= line_count {
            buf.current = line_count - 1;
            buf.current_x = buf.lines[buf.current].data.len();
        } else {
            let target = buf.placewewant;
            buf.current += mustmove;
            buf.current_x = chars::actual_x(&buf.lines[buf.current].data, target, tabsize);
        }

        self.refresh_needed = true;
    }

    /// Move to the previous word.
    pub fn to_prev_word(&mut self) {
        let tabsize = self.tabsize;
        let buf = &mut self.buffers[self.current_buf];
        let word_chars = &None; // TODO: pull from editor state
        let punct_as_letters = false;

        let mut seen_a_word = false;
        let mut step_forward = false;

        loop {
            if buf.current_x == 0 {
                if buf.current == 0 {
                    break;
                }
                buf.current -= 1;
                buf.current_x = buf.lines[buf.current].data.len();
            }

            buf.current_x = chars::step_left(&buf.lines[buf.current].data, buf.current_x);

            if let Some(c) = buf.lines[buf.current].data[buf.current_x..].chars().next() {
                if chars::is_word_char(c, punct_as_letters, word_chars) {
                    seen_a_word = true;
                    if buf.current_x == 0 {
                        break;
                    }
                } else if chars::is_zerowidth(c) {
                    // skip
                } else if seen_a_word {
                    step_forward = true;
                    break;
                }
            }
        }

        if step_forward {
            buf.current_x = chars::step_right(&buf.lines[buf.current].data, buf.current_x);
        }

        buf.placewewant = chars::wideness(
            &buf.lines[buf.current].data,
            buf.current_x,
            tabsize,
        );
        self.refresh_needed = true;
    }

    /// Move to the next word.
    pub fn to_next_word(&mut self) {
        let tabsize = self.tabsize;
        let buf = &mut self.buffers[self.current_buf];
        let word_chars = &None;
        let punct_as_letters = false;
        let line_count = buf.lines.len();

        let mut seen_space = if let Some(c) = buf.lines[buf.current].data[buf.current_x..].chars().next() {
            !chars::is_word_char(c, punct_as_letters, word_chars)
        } else {
            true
        };

        loop {
            let at_eol = buf.current_x >= buf.lines[buf.current].data.len();
            if at_eol {
                if buf.current >= line_count - 1 {
                    break;
                }
                buf.current += 1;
                buf.current_x = 0;
                seen_space = true;
            } else {
                buf.current_x = chars::step_right(&buf.lines[buf.current].data, buf.current_x);
            }

            if let Some(c) = buf.lines[buf.current].data[buf.current_x..].chars().next() {
                if chars::is_zerowidth(c) {
                    // skip
                } else if !chars::is_word_char(c, punct_as_letters, word_chars) {
                    seen_space = true;
                } else if seen_space {
                    break;
                }
            }
        }

        buf.placewewant = chars::wideness(
            &buf.lines[buf.current].data,
            buf.current_x,
            tabsize,
        );
        self.refresh_needed = true;
    }

    /// Move to the preceding block of text.
    pub fn to_prev_block(&mut self) {
        let buf = &mut self.buffers[self.current_buf];
        let mut is_text = false;
        let mut seen_text = false;

        while buf.current > 0 && (!seen_text || is_text) {
            buf.current -= 1;
            is_text = !chars::white_string(&buf.lines[buf.current].data);
            seen_text = seen_text || is_text;
        }

        if seen_text && buf.current < buf.lines.len() - 1
            && chars::white_string(&buf.lines[buf.current].data)
        {
            buf.current += 1;
        }

        buf.current_x = 0;
        buf.placewewant = 0;
        self.refresh_needed = true;
    }

    /// Move to the next block of text.
    pub fn to_next_block(&mut self) {
        let buf = &mut self.buffers[self.current_buf];
        let line_count = buf.lines.len();
        let mut is_white = chars::white_string(&buf.lines[buf.current].data);
        let mut seen_white = is_white;

        while buf.current < line_count - 1 && (!seen_white || is_white) {
            buf.current += 1;
            is_white = chars::white_string(&buf.lines[buf.current].data);
            seen_white = seen_white || is_white;
        }

        buf.current_x = 0;
        buf.placewewant = 0;
        self.refresh_needed = true;
    }

    /// Scroll up one line without moving cursor textwise.
    pub fn do_scroll_up(&mut self) {
        let editwinrows = self.editwinrows;
        let buf = &mut self.buffers[self.current_buf];
        if buf.edittop == 0 {
            return;
        }
        buf.edittop -= 1;
        if buf.cursor_row < editwinrows as isize - 1 {
            buf.cursor_row += 1;
        } else {
            self.do_up();
        }
        self.refresh_needed = true;
    }

    /// Scroll down one line without moving cursor textwise.
    pub fn do_scroll_down(&mut self) {
        let line_count = self.buffers[self.current_buf].lines.len();
        let buf = &mut self.buffers[self.current_buf];
        if buf.edittop >= line_count - 1 {
            return;
        }
        buf.edittop += 1;
        if buf.cursor_row > 0 {
            buf.cursor_row -= 1;
        } else {
            self.do_down();
        }
        self.refresh_needed = true;
    }
}
