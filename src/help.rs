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


/// Help viewer. Port of nano's help.c.

use crate::definitions::*;
use crate::editor::Editor;

use std::io;

#[cfg(feature = "help")]
impl Editor {
    /// Display the help screen (^G).
    pub fn do_help(&mut self) -> io::Result<()> {
        let help_text = self.build_help_text();
        let help_lines: Vec<&str> = help_text.lines().collect();

        let saved_menu = self.currmenu;
        self.inhelp = true;
        let mut top = 0usize;

        loop {
            // Draw help screen.
            self.draw_help_screen(&help_lines, top)?;

            let keycode = self.tio.get_kbinput()?;

            match keycode {
                KeyCode::Escape | KeyCode::Ctrl('c') | KeyCode::Ctrl('g') | KeyCode::F(1) => {
                    break;
                }
                KeyCode::Ctrl('x') | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    break;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if top > 0 {
                        top -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if top + self.editwinrows < help_lines.len() {
                        top += 1;
                    }
                }
                KeyCode::PageUp => {
                    top = top.saturating_sub(self.editwinrows);
                }
                KeyCode::PageDown => {
                    if top + self.editwinrows < help_lines.len() {
                        top = (top + self.editwinrows).min(
                            help_lines.len().saturating_sub(self.editwinrows),
                        );
                    }
                }
                KeyCode::Home => {
                    top = 0;
                }
                KeyCode::End => {
                    top = help_lines.len().saturating_sub(self.editwinrows);
                }
                _ => {}
            }
        }

        self.inhelp = false;
        self.currmenu = saved_menu;
        self.refresh_needed = true;
        Ok(())
    }

    /// Build the help text string for the current menu context.
    fn build_help_text(&self) -> String {
        let mut text = String::new();

        text.push_str(" rano help text\n\n");
        text.push_str(" The main shortcuts are:\n\n");

        // List all bindings for the main menu.
        for binding in &self.keybindings {
            if binding.menus.contains(Menu::MAIN) {
                // Find the function tag.
                let tag = self
                    .func_table
                    .iter()
                    .find(|f| f.func == binding.func)
                    .map(|f| f.tag)
                    .unwrap_or("???");

                text.push_str(&format!("   {:>8}   {}\n", binding.keystr, tag));
            }
        }

        text.push_str("\n The editor is a Rust port of GNU nano.\n");
        text.push_str(" For more information, see the nano documentation.\n");

        text
    }

    /// Draw the help screen on the terminal.
    fn draw_help_screen(&mut self, lines: &[&str], top: usize) -> io::Result<()> {
        // Title bar
        let title = " rano Help";
        let padded = format!("{:<width$}", title, width = self.term_cols);
        self.tio.move_cursor(0, 0)?;
        self.tio.print_reversed(&padded)?;

        // Help content
        let visible_rows = self.editwinrows;
        for row in 0..visible_rows {
            self.tio.move_cursor((row + 1) as u16, 0)?;
            self.tio.clear_to_eol()?;
            let line_idx = top + row;
            if line_idx < lines.len() {
                let display = if lines[line_idx].len() > self.term_cols {
                    &lines[line_idx][..self.term_cols]
                } else {
                    lines[line_idx]
                };
                self.tio.print_str(display)?;
            }
        }

        // Status bar
        let status_row = self.term_rows
            - if self.flags.contains(EditorFlags::NO_HELP) {
                1
            } else {
                3
            };
        self.tio.move_cursor(status_row as u16, 0)?;
        let status = format!(
            "[ line {}/{} ({}%) ]",
            top + 1,
            lines.len(),
            if lines.is_empty() {
                0
            } else {
                (top + 1) * 100 / lines.len()
            }
        );
        let padded = format!("{:<width$}", status, width = self.term_cols);
        self.tio.print_reversed(&padded)?;

        // Help lines at the bottom
        if !self.flags.contains(EditorFlags::NO_HELP) {
            let help_row1 = self.term_rows - 2;
            let help_row2 = self.term_rows - 1;

            self.tio.move_cursor(help_row1 as u16, 0)?;
            self.tio.clear_to_eol()?;
            self.tio.print_reversed("^X")?;
            self.tio.print_str(" Exit Help   ")?;

            self.tio.move_cursor(help_row2 as u16, 0)?;
            self.tio.clear_to_eol()?;
        }

        self.tio.flush()?;
        Ok(())
    }
}

/// When the help feature is not enabled, provide a stub.
#[cfg(not(feature = "help"))]
impl Editor {
    pub fn do_help(&mut self) -> io::Result<()> {
        self.statusline(MessageType::Ahem, "Help is not available");
        Ok(())
    }
}
