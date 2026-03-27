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


/// Cut, copy, and paste operations. Port of nano's cut.c.

use crate::chars;
use crate::definitions::*;
use crate::editor::Editor;

use std::process::{Command, Stdio};
use std::io::Write as IoWrite;

/// Copy text to the system clipboard.
fn clipboard_copy(text: &str) {
    #[cfg(target_os = "macos")]
    let result = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(text.as_bytes())?;
            }
            child.wait()
        });

    #[cfg(not(target_os = "macos"))]
    let result = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(ref mut stdin) = child.stdin {
                stdin.write_all(text.as_bytes())?;
            }
            child.wait()
        });

    let _ = result;
}

/// Read text from the system clipboard.
fn clipboard_paste() -> Option<String> {
    #[cfg(target_os = "macos")]
    let output = Command::new("pbpaste").output().ok()?;

    #[cfg(not(target_os = "macos"))]
    let output = Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

impl Editor {
    /// Cut the current line (or marked region) into the cutbuffer.
    pub fn cut_text(&mut self) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            self.statusline(MessageType::Ahem, "Key invalid in view mode");
            return;
        }

        let buf = &self.buffers[self.current_buf];
        // Check if there's anything to cut
        if buf.current >= buf.lines.len() - 1 && buf.lines[buf.current].data.is_empty() && buf.mark.is_none() {
            self.statusline(MessageType::Ahem, "Nothing was cut");
            return;
        }

        if !self.keep_cutbuffer {
            self.cutbuffer.clear();
        }

        let buf = &self.buffers[self.current_buf];
        if let Some(mark_line) = buf.mark {
            // For drag selections, the endpoint is drag_end; otherwise it's the cursor
            let (end_line, end_x) = if buf.softmark {
                if let Some(de) = buf.drag_end {
                    (de, buf.drag_end_x)
                } else {
                    (buf.current, buf.current_x)
                }
            } else {
                (buf.current, buf.current_x)
            };

            let (top, top_x, bot, bot_x) = if mark_line < end_line
                || (mark_line == end_line && buf.mark_x <= end_x)
            {
                (mark_line, buf.mark_x, end_line, end_x)
            } else {
                (end_line, end_x, mark_line, buf.mark_x)
            };

            self.cut_region(top, top_x, bot, bot_x);
            let buf = self.current_buffer_mut();
            buf.mark = None;
            buf.drag_end = None;
            self.keep_cutbuffer = false;
        } else {
            // Cut the entire current line
            let current = self.current_buffer().current;
            let line_count = self.current_buffer().lines.len();

            if current < line_count - 1 {
                let cut_line = self.current_buffer().lines[current].clone();
                self.cutbuffer.push(cut_line);
                self.cutbuffer.push(Line::new(String::new(), 0));
                self.current_buffer_mut().lines.remove(current);
                if self.current_buffer().current >= self.current_buffer().lines.len() {
                    self.current_buffer_mut().current = self.current_buffer().lines.len() - 1;
                }
                self.current_buffer_mut().current_x = 0;
            } else {
                // Last line: cut content but leave empty line
                let cut_line = self.current_buffer().lines[current].clone();
                self.cutbuffer.push(cut_line);
                self.current_buffer_mut().lines[current].data.clear();
                self.current_buffer_mut().current_x = 0;
            }

            self.current_buffer_mut().renumber_from(current);
            self.keep_cutbuffer = true;
        }

        // Also copy to system clipboard
        let clipboard_text = self.cutbuffer_to_string();
        clipboard_copy(&clipboard_text);

        self.set_modified();
        self.confirm_margin();
        self.refresh_needed = true;
    }

    /// Cut a region from (top, top_x) to (bot, bot_x).
    fn cut_region(&mut self, top: usize, top_x: usize, bot: usize, bot_x: usize) {
        if top == bot {
            // Single line region
            let extracted = self.buffers[self.current_buf].lines[top].data[top_x..bot_x].to_string();
            self.cutbuffer.push(Line::new(extracted, 0));
            self.buffers[self.current_buf].lines[top].data.drain(top_x..bot_x);
            self.buffers[self.current_buf].current = top;
            self.buffers[self.current_buf].current_x = top_x;
        } else {
            // Multi-line region
            // First line: take from top_x to end
            let first_part = self.buffers[self.current_buf].lines[top].data[top_x..].to_string();
            self.cutbuffer.push(Line::new(first_part, 0));

            // Middle lines: take entirely
            for i in (top + 1)..bot {
                self.cutbuffer.push(self.buffers[self.current_buf].lines[i].clone());
            }

            // Last line: take from start to bot_x
            let last_part = self.buffers[self.current_buf].lines[bot].data[..bot_x].to_string();
            self.cutbuffer.push(Line::new(last_part, 0));

            // Merge top line's before-portion with bot line's after-portion
            let after_bot = self.buffers[self.current_buf].lines[bot].data[bot_x..].to_string();
            self.buffers[self.current_buf].lines[top].data.truncate(top_x);
            self.buffers[self.current_buf].lines[top].data.push_str(&after_bot);

            // Remove lines from top+1 through bot
            for _ in (top + 1)..=bot {
                self.buffers[self.current_buf].lines.remove(top + 1);
            }

            self.buffers[self.current_buf].current = top;
            self.buffers[self.current_buf].current_x = top_x;
            self.buffers[self.current_buf].renumber_from(top);
        }
    }

    /// Copy the current line (or marked region) into the cutbuffer.
    pub fn copy_text(&mut self) {
        self.cutbuffer.clear();

        let buf = &self.buffers[self.current_buf];

        if let Some(mark_line) = buf.mark {
            // For drag selections, the endpoint is drag_end; otherwise it's the cursor
            let (end_line, end_x) = if buf.softmark {
                if let Some(de) = buf.drag_end {
                    (de, buf.drag_end_x)
                } else {
                    (buf.current, buf.current_x)
                }
            } else {
                (buf.current, buf.current_x)
            };

            let (top, top_x, bot, bot_x) = if mark_line < end_line
                || (mark_line == end_line && buf.mark_x <= end_x)
            {
                (mark_line, buf.mark_x, end_line, end_x)
            } else {
                (end_line, end_x, mark_line, buf.mark_x)
            };

            if top == bot && top_x == bot_x {
                self.statusline(MessageType::Ahem, "Copied nothing");
                return;
            }

            // Copy the region without modifying the buffer
            if top == bot {
                let text = buf.lines[top].data[top_x..bot_x].to_string();
                self.cutbuffer.push(Line::new(text, 0));
            } else {
                self.cutbuffer.push(Line::new(buf.lines[top].data[top_x..].to_string(), 0));
                for i in (top + 1)..bot {
                    self.cutbuffer.push(buf.lines[i].clone());
                }
                self.cutbuffer.push(Line::new(buf.lines[bot].data[..bot_x].to_string(), 0));
            }

            let buf = self.current_buffer_mut();
            buf.mark = None;
            buf.drag_end = None;
        } else {
            // Copy the current line
            let current = buf.current;
            self.cutbuffer.push(buf.lines[current].clone());
            self.cutbuffer.push(Line::new(String::new(), 0));
        }

        // Also copy to system clipboard
        let clipboard_text = self.cutbuffer_to_string();
        clipboard_copy(&clipboard_text);

        self.statusline(MessageType::Info, "Copied text");
        self.refresh_needed = true;
    }

    /// Convert the cutbuffer to a single string for clipboard use.
    fn cutbuffer_to_string(&self) -> String {
        let mut result = String::new();
        for (i, line) in self.cutbuffer.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&line.data);
        }
        result
    }

    /// Paste the cutbuffer contents at the cursor position.
    pub fn paste_text(&mut self) {
        if self.flags.contains(EditorFlags::VIEW_MODE) {
            self.statusline(MessageType::Ahem, "Key invalid in view mode");
            return;
        }

        // If internal cutbuffer is empty, try system clipboard
        if self.cutbuffer.is_empty() {
            if let Some(text) = clipboard_paste() {
                if !text.is_empty() {
                    self.cutbuffer.clear();
                    let mut lineno = 0;
                    for line_str in text.split('\n') {
                        self.cutbuffer.push(Line::new(line_str.to_string(), lineno));
                        lineno += 1;
                    }
                }
            }
        }

        if self.cutbuffer.is_empty() {
            self.statusline(MessageType::Ahem, "Cutbuffer is empty");
            return;
        }

        self.add_undo(UndoType::Paste);

        let cutbuf = self.cutbuffer.clone();
        let tabsize = self.tabsize;
        let buf = self.current_buffer_mut();
        let current_idx = buf.current;
        let current_x = buf.current_x;

        if cutbuf.len() == 1 {
            // Single-line paste: insert text at cursor
            let text = &cutbuf[0].data;
            buf.lines[current_idx].data.insert_str(current_x, text);
            buf.current_x = current_x + text.len();
        } else {
            // Multi-line paste
            let after_cursor = buf.lines[current_idx].data[current_x..].to_string();
            buf.lines[current_idx].data.truncate(current_x);
            buf.lines[current_idx].data.push_str(&cutbuf[0].data);

            let mut insert_idx = current_idx + 1;
            for i in 1..cutbuf.len() - 1 {
                let line = Line::new(cutbuf[i].data.clone(), insert_idx + 1);
                buf.lines.insert(insert_idx, line);
                insert_idx += 1;
            }

            // Last cutbuffer line + after_cursor
            let last = &cutbuf[cutbuf.len() - 1];
            let final_x = last.data.len();
            let final_line = Line::new(format!("{}{}", last.data, after_cursor), insert_idx + 1);
            buf.lines.insert(insert_idx, final_line);

            buf.current = insert_idx;
            buf.current_x = final_x;
            buf.renumber_from(current_idx);
        }

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
