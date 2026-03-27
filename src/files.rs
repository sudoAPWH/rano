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


/// File operations. Port of nano's files.c.

use crate::definitions::*;
use crate::editor::Editor;
use crate::utils;

use std::fs;
use std::io::{self, Write as IoWrite};
use std::path::Path;

impl Editor {
    /// Open a file into a new buffer.
    pub fn open_buffer(&mut self, filename: &str) -> io::Result<()> {
        let expanded = utils::real_dir_from_tilde(filename);
        let mut buf = OpenBuffer::new();
        buf.filename = expanded.clone();

        let path = Path::new(&expanded);

        if path.exists() {
            let content = fs::read_to_string(path).map_err(|e| {
                io::Error::new(e.kind(), format!("{}: {}", expanded, e))
            })?;

            buf.lines.clear();

            // Detect format
            let format = if content.contains("\r\n") {
                FormatType::Dos
            } else if content.contains('\r') {
                FormatType::Mac
            } else {
                FormatType::Unix
            };
            buf.format = format;

            // Normalize line endings
            let normalized = content.replace("\r\n", "\n").replace('\r', "\n");

            let mut lineno = 1;
            for line_str in normalized.split('\n') {
                buf.lines.push(Line::new(line_str.to_string(), lineno));
                lineno += 1;
            }

            // If the file ended with a newline, the split produces an extra empty element.
            // Keep it as the "magic line" (like nano does).
            if buf.lines.is_empty() {
                buf.lines.push(Line::new(String::new(), 1));
            }

            // If the last line is empty and there are at least 2 lines,
            // it represents the trailing newline — keep it as magic line.

            buf.totsize = content.len();
        } else {
            // New file
            buf.lines = vec![Line::new(String::new(), 1)];
            buf.totsize = 0;
        }

        buf.modified = false;
        buf.current = 0;
        buf.current_x = 0;
        buf.edittop = 0;

        self.buffers.push(buf);
        self.current_buf = self.buffers.len() - 1;

        let name = self.buffers[self.current_buf].filename.clone();
        let lines = self.buffers[self.current_buf].lines.len();
        if path.exists() {
            self.statusline(
                MessageType::Info,
                &format!("Read {} lines ({})", lines, utils::tail(&name)),
            );
        } else {
            self.statusline(
                MessageType::Info,
                &format!("New File ({})", utils::tail(&name)),
            );
        }

        Ok(())
    }

    /// Write the current buffer to disk.
    pub fn write_file(&mut self, filename: &str) -> io::Result<bool> {
        let expanded = utils::real_dir_from_tilde(filename);
        let path = Path::new(&expanded);

        // Backup if requested
        if self.flags.contains(EditorFlags::MAKE_BACKUP) && path.exists() {
            let backup = format!("{}~", expanded);
            let _ = fs::copy(path, &backup);
        }

        let line_ending = match self.buffers[self.current_buf].format {
            FormatType::Dos => "\r\n",
            FormatType::Mac => "\r",
            _ => "\n",
        };

        let mut file = fs::File::create(path)?;
        let buf = &self.buffers[self.current_buf];
        let last_idx = buf.lines.len() - 1;

        for (i, line) in buf.lines.iter().enumerate() {
            file.write_all(line.data.as_bytes())?;
            // Write line ending for all lines except possibly the very last
            // if it's the magic empty line and NO_NEWLINES is set
            if i < last_idx {
                file.write_all(line_ending.as_bytes())?;
            } else if !line.data.is_empty() || !self.flags.contains(EditorFlags::NO_NEWLINES) {
                file.write_all(line_ending.as_bytes())?;
            }
        }

        file.flush()?;

        let buf = self.current_buffer_mut();
        buf.modified = false;
        buf.last_saved_index = buf.undo_index;

        let line_count = buf.lines.len();
        let name = buf.filename.clone();
        self.statusline(
            MessageType::Info,
            &format!("Wrote {} lines ({})", line_count, utils::tail(&name)),
        );

        Ok(true)
    }

    /// Handle the Write Out command (^O).
    pub fn do_writeout(&mut self) -> io::Result<()> {
        let current_name = self.buffers[self.current_buf].filename.clone();
        let prompt_msg = if current_name.is_empty() {
            "File Name to Write"
        } else {
            "File Name to Write"
        };

        // Show prompt pre-filled with current filename (like nano).
        let name = self.do_prompt_with_default(prompt_msg, &current_name)?;
        if name.is_empty() {
            self.statusline(MessageType::Info, "Cancelled");
            return Ok(());
        }
        self.buffers[self.current_buf].filename = name.clone();
        self.write_file(&name)?;
        self.refresh_needed = true;
        Ok(())
    }

    /// Handle the Save command (^S).
    pub fn do_savefile(&mut self) -> io::Result<()> {
        let filename = self.buffers[self.current_buf].filename.clone();
        if filename.is_empty() {
            return self.do_writeout();
        }
        self.write_file(&filename)?;
        self.refresh_needed = true;
        Ok(())
    }

    /// Handle the Exit command (^X).
    pub fn do_exit(&mut self) -> io::Result<()> {
        if self.buffers[self.current_buf].modified {
            let response = self.ask_yes_no("Save modified buffer? ")?;
            match response {
                AskResult::Yes => {
                    self.do_savefile()?;
                    if !self.buffers[self.current_buf].modified {
                        self.close_current_buffer();
                    }
                }
                AskResult::No => {
                    self.close_current_buffer();
                }
                AskResult::Cancel => {
                    self.statusline(MessageType::Info, "Cancelled");
                }
                _ => {}
            }
        } else {
            self.close_current_buffer();
        }
        self.refresh_needed = true;
        Ok(())
    }

    /// Close the current buffer. If it's the last one, exit the editor.
    fn close_current_buffer(&mut self) {
        if self.buffers.len() <= 1 {
            self.running = false;
        } else {
            self.buffers.remove(self.current_buf);
            if self.current_buf >= self.buffers.len() {
                self.current_buf = self.buffers.len() - 1;
            }
            self.confirm_margin();
        }
    }

    /// Insert a file into the current buffer at the cursor position.
    pub fn do_insertfile(&mut self) -> io::Result<()> {
        let filename = self.do_prompt("File to insert: ")?;
        if filename.is_empty() {
            self.statusline(MessageType::Info, "Cancelled");
            return Ok(());
        }

        let expanded = utils::real_dir_from_tilde(&filename);
        let content = match fs::read_to_string(&expanded) {
            Ok(c) => c,
            Err(e) => {
                self.statusline(MessageType::Alert, &format!("Error reading {}: {}", filename, e));
                return Ok(());
            }
        };

        let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
        let insert_lines: Vec<&str> = normalized.split('\n').collect();

        if insert_lines.is_empty() {
            return Ok(());
        }

        let buf = self.current_buffer_mut();
        let current_idx = buf.current;
        let current_x = buf.current_x;

        // Split current line at cursor
        let after_cursor = buf.lines[current_idx].data[current_x..].to_string();
        buf.lines[current_idx].data.truncate(current_x);

        // Append first inserted line to current line
        buf.lines[current_idx].data.push_str(insert_lines[0]);

        // Insert middle lines
        let mut insert_idx = current_idx + 1;
        for i in 1..insert_lines.len() {
            let line = Line::new(insert_lines[i].to_string(), insert_idx + 1);
            buf.lines.insert(insert_idx, line);
            insert_idx += 1;
        }

        // Append the after-cursor text to the last inserted line
        let last_inserted = insert_idx - 1;
        buf.lines[last_inserted].data.push_str(&after_cursor);

        buf.current = last_inserted;
        buf.current_x = buf.lines[last_inserted].data.len() - after_cursor.len();
        buf.renumber_from(current_idx);

        self.set_modified();
        self.confirm_margin();
        self.refresh_needed = true;

        self.statusline(
            MessageType::Info,
            &format!("Inserted {} lines", insert_lines.len()),
        );

        Ok(())
    }
}
