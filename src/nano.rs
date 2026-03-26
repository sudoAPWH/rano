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


/// Signal handling and miscellaneous nano-level functions.
/// Port of parts of nano's nano.c that don't fit elsewhere.

use crate::definitions::*;
use crate::editor::Editor;

use std::io;

impl Editor {
    /// Handle terminal resize (SIGWINCH equivalent).
    pub fn handle_resize(&mut self) -> io::Result<()> {
        let (cols, rows) = crossterm::terminal::size()?;
        self.term_rows = rows as usize;
        self.term_cols = cols as usize;

        let help_rows = if self.flags.contains(EditorFlags::NO_HELP) {
            0
        } else {
            2
        };
        self.editwinrows = self.term_rows.saturating_sub(2 + help_rows);
        self.confirm_margin();
        self.refresh_needed = true;
        Ok(())
    }

    /// Count lines, words, and characters in the current buffer.
    pub fn count_lines_words_characters(&mut self) {
        let buf = &self.buffers[self.current_buf];
        let lines = buf.lines.len();
        let mut words = 0usize;
        let mut chars = 0usize;

        for line in &buf.lines {
            chars += line.data.chars().count();
            chars += 1; // newline
            let mut in_word = false;
            for c in line.data.chars() {
                if c.is_whitespace() {
                    in_word = false;
                } else {
                    if !in_word {
                        words += 1;
                    }
                    in_word = true;
                }
            }
        }

        // Subtract the trailing newline of the last line if it's the magic line.
        if buf.lines.last().map_or(false, |l| l.data.is_empty()) && lines > 1 {
            chars = chars.saturating_sub(1);
        }

        self.statusline(
            MessageType::Info,
            &format!("Lines: {}  Words: {}  Chars: {}", lines, words, chars),
        );
        self.refresh_needed = true;
    }

    /// Chop the previous word (Alt+Backspace).
    pub fn chop_previous_word(&mut self) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            self.statusline(MessageType::Ahem, "Key invalid in view mode");
            return;
        }

        let buf = &self.buffers[self.current_buf];
        if buf.current_x == 0 {
            // At start of line — just do a backspace (joins with previous line).
            self.do_backspace();
            return;
        }

        // Find the start of the previous word.
        let line = &buf.lines[buf.current].data;
        let mut pos = buf.current_x;

        // Skip any spaces before cursor.
        while pos > 0 && line.as_bytes().get(pos - 1).map_or(false, |&b| b == b' ' || b == b'\t')
        {
            pos -= 1;
        }
        // Skip the word.
        while pos > 0
            && line
                .as_bytes()
                .get(pos - 1)
                .map_or(false, |&b| b != b' ' && b != b'\t')
        {
            pos -= 1;
        }

        // Delete from pos to current_x.
        let current_x = self.buffers[self.current_buf].current_x;
        let current = self.buffers[self.current_buf].current;
        self.buffers[self.current_buf].lines[current]
            .data
            .drain(pos..current_x);
        self.buffers[self.current_buf].current_x = pos;

        self.set_modified();
        self.refresh_needed = true;
    }

    /// Chop the next word (Alt+Delete equivalent).
    pub fn chop_next_word(&mut self) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            self.statusline(MessageType::Ahem, "Key invalid in view mode");
            return;
        }

        let buf = &self.buffers[self.current_buf];
        let line_len = buf.lines[buf.current].data.len();

        if buf.current_x >= line_len {
            // At end of line — just do a delete (joins with next line).
            self.do_delete();
            return;
        }

        let line = &buf.lines[buf.current].data;
        let start = buf.current_x;
        let mut pos = start;

        // Skip the current word.
        while pos < line.len()
            && line
                .as_bytes()
                .get(pos)
                .map_or(false, |&b| b != b' ' && b != b'\t')
        {
            pos += 1;
        }
        // Skip any spaces after the word.
        while pos < line.len()
            && line
                .as_bytes()
                .get(pos)
                .map_or(false, |&b| b == b' ' || b == b'\t')
        {
            pos += 1;
        }

        let current = self.buffers[self.current_buf].current;
        self.buffers[self.current_buf].lines[current]
            .data
            .drain(start..pos);

        self.set_modified();
        self.refresh_needed = true;
    }
}
