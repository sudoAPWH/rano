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


/// Prompt and status bar interactions. Port of nano's prompt.c.

use crate::chars;
use crate::definitions::*;
use crate::editor::Editor;

use std::io;

impl Editor {
    /// Display a prompt with a pre-filled default value.
    pub fn do_prompt_with_default(&mut self, msg: &str, default: &str) -> io::Result<String> {
        let mut answer = default.to_string();
        let mut typing_x: usize = answer.len();
        let saved_menu = self.currmenu;

        loop {
            self.draw_prompt_bar(msg, &answer, typing_x)?;
            let keycode = self.tio.get_kbinput()?;

            match keycode {
                KeyCode::Enter => {
                    self.currmenu = saved_menu;
                    return Ok(answer);
                }
                KeyCode::Escape | KeyCode::Ctrl('c') => {
                    self.currmenu = saved_menu;
                    return Ok(String::new());
                }
                KeyCode::Backspace => {
                    if typing_x > 0 {
                        let prev_x = crate::chars::step_left(&answer, typing_x);
                        answer.drain(prev_x..typing_x);
                        typing_x = prev_x;
                    }
                }
                KeyCode::Delete => {
                    if typing_x < answer.len() {
                        let next_x = crate::chars::step_right(&answer, typing_x);
                        answer.drain(typing_x..next_x);
                    }
                }
                KeyCode::Left => {
                    if typing_x > 0 {
                        typing_x = crate::chars::step_left(&answer, typing_x);
                    }
                }
                KeyCode::Right => {
                    if typing_x < answer.len() {
                        typing_x = crate::chars::step_right(&answer, typing_x);
                    }
                }
                KeyCode::Home => { typing_x = 0; }
                KeyCode::End => { typing_x = answer.len(); }
                KeyCode::Ctrl('k') => {
                    if typing_x >= answer.len() { typing_x = 0; }
                    answer.truncate(typing_x);
                }
                KeyCode::Char(c) => {
                    answer.insert(typing_x, c);
                    typing_x += c.len_utf8();
                }
                KeyCode::Tab => {
                    answer.insert(typing_x, '\t');
                    typing_x += 1;
                }
                _ => {}
            }
        }
    }

    /// Display a prompt on the status bar and get a text response.
    /// Returns the entered string, or empty string if cancelled.
    pub fn do_prompt(&mut self, msg: &str) -> io::Result<String> {
        self.do_prompt_with_default(msg, "")
    }

    /// Ask a Yes/No question and return the result.
    pub fn ask_yes_no(&mut self, question: &str) -> io::Result<AskResult> {
        loop {
            // Draw the question
            let row = self.term_rows - if self.flags.contains(EditorFlags::NO_HELP) { 1 } else { 3 };
            self.tio.move_cursor(row as u16, 0)?;
            let padded = format!("{:<width$}", question, width = self.term_cols);
            self.tio.print_reversed(&padded)?;

            // Draw help shortcuts for Yes/No
            if !self.flags.contains(EditorFlags::NO_HELP) {
                let row1 = self.term_rows - 2;
                let row2 = self.term_rows - 1;
                let width = 16.min(self.term_cols / 2);

                self.tio.move_cursor(row1 as u16, 0)?;
                self.tio.clear_to_eol()?;
                self.tio.print_reversed(" Y")?;
                self.tio.print_str(&format!(" {:<width$}", "Yes", width = width - 3))?;

                self.tio.move_cursor(row2 as u16, 0)?;
                self.tio.clear_to_eol()?;
                self.tio.print_reversed(" N")?;
                self.tio.print_str(&format!(" {:<width$}", "No", width = width - 3))?;

                self.tio.move_cursor(row2 as u16, width as u16)?;
                self.tio.print_reversed("^C")?;
                self.tio.print_str(&format!(" {:<width$}", "Cancel", width = width - 3))?;
            }

            self.tio.flush()?;

            let keycode = self.tio.get_kbinput()?;

            match keycode {
                KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(AskResult::Yes),
                KeyCode::Char('n') | KeyCode::Char('N') => return Ok(AskResult::No),
                KeyCode::Char('a') | KeyCode::Char('A') => return Ok(AskResult::All),
                KeyCode::Ctrl('c') | KeyCode::Escape => return Ok(AskResult::Cancel),
                _ => {
                    // beep and loop
                }
            }
        }
    }

    /// Draw the prompt bar with the message, answer text, and cursor.
    fn draw_prompt_bar(&mut self, msg: &str, answer: &str, typing_x: usize) -> io::Result<()> {
        let row = self.term_rows - if self.flags.contains(EditorFlags::NO_HELP) { 1 } else { 3 };
        self.tio.move_cursor(row as u16, 0)?;

        let prompt_text = format!("{}: {}", msg, answer);
        let padded = format!("{:<width$}", prompt_text, width = self.term_cols);
        self.tio.print_reversed(&padded)?;

        // Position cursor within the answer
        let prompt_len = msg.len() + 2; // ": "
        let cursor_col = prompt_len + chars::wideness(answer, typing_x, self.tabsize);
        self.tio.move_cursor(row as u16, cursor_col as u16)?;
        self.tio.show_cursor()?;
        self.tio.flush()?;

        Ok(())
    }
}
