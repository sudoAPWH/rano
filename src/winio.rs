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


/// Terminal I/O layer. Port of nano's winio.c using crossterm.

use crate::chars;
use crate::definitions::*;
use crate::editor::Editor;
use crate::utils;

use crossterm::{
    cursor,
    event::{self, Event, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    execute, queue,
    style::{self, Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write, Stdout, stdout};
use std::time::Duration;

/// Terminal I/O handler.
pub struct TermIO {
    stdout: Stdout,
    in_raw_mode: bool,
}

impl TermIO {
    pub fn new() -> io::Result<Self> {
        Ok(TermIO {
            stdout: stdout(),
            in_raw_mode: false,
        })
    }

    /// Enter raw mode and prepare the terminal.
    pub fn enter_raw_mode(&mut self) -> io::Result<()> {
        if !self.in_raw_mode {
            terminal::enable_raw_mode()?;
            self.in_raw_mode = true;
        }
        Ok(())
    }

    /// Set up the screen (alternate screen, mouse, etc.).
    pub fn setup_screen(&mut self) -> io::Result<()> {
        execute!(
            self.stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(ClearType::All),
        )?;

        // Always enable mouse capture for scroll wheel support.
        // The -m flag controls click-to-position behavior, handled in the editor.
        execute!(self.stdout, event::EnableMouseCapture)?;

        // Enable bracketed paste
        execute!(self.stdout, event::EnableBracketedPaste)?;

        Ok(())
    }

    /// Clean up terminal state.
    pub fn cleanup(&mut self) -> io::Result<()> {
        execute!(
            self.stdout,
            event::DisableBracketedPaste,
            event::DisableMouseCapture,
            cursor::Show,
            terminal::LeaveAlternateScreen,
        )?;
        if self.in_raw_mode {
            terminal::disable_raw_mode()?;
            self.in_raw_mode = false;
        }
        Ok(())
    }

    /// Read a single keypress and translate it to our KeyCode.
    pub fn get_kbinput(&mut self) -> io::Result<KeyCode> {
        loop {
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key_event) => return Ok(translate_key(key_event)),
                    Event::Resize(w, h) => {
                        return Ok(KeyCode::Special(SpecialKey::WindowResized));
                    }
                    Event::Mouse(mouse_event) => {
                        match mouse_event.kind {
                            MouseEventKind::ScrollUp => {
                                return Ok(KeyCode::Special(SpecialKey::MouseScrollUp));
                            }
                            MouseEventKind::ScrollDown => {
                                return Ok(KeyCode::Special(SpecialKey::MouseScrollDown));
                            }
                            _ => continue,
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    /// Move the cursor to a given position.
    pub fn move_cursor(&mut self, row: u16, col: u16) -> io::Result<()> {
        queue!(self.stdout, cursor::MoveTo(col, row))?;
        Ok(())
    }

    /// Show the cursor.
    pub fn show_cursor(&mut self) -> io::Result<()> {
        queue!(self.stdout, cursor::Show)?;
        Ok(())
    }

    /// Hide the cursor.
    pub fn hide_cursor(&mut self) -> io::Result<()> {
        queue!(self.stdout, cursor::Hide)?;
        Ok(())
    }

    /// Clear the entire screen.
    pub fn clear_screen(&mut self) -> io::Result<()> {
        queue!(self.stdout, terminal::Clear(ClearType::All))?;
        Ok(())
    }

    /// Clear to end of line.
    pub fn clear_to_eol(&mut self) -> io::Result<()> {
        queue!(self.stdout, terminal::Clear(ClearType::UntilNewLine))?;
        Ok(())
    }

    /// Print a string at the current cursor position.
    pub fn print_str(&mut self, s: &str) -> io::Result<()> {
        queue!(self.stdout, Print(s))?;
        Ok(())
    }

    /// Print a string with reverse video (for titlebar, statusbar).
    pub fn print_reversed(&mut self, s: &str) -> io::Result<()> {
        queue!(
            self.stdout,
            SetAttribute(Attribute::Reverse),
            Print(s),
            SetAttribute(Attribute::Reset),
        )?;
        Ok(())
    }

    /// Print a string with bold.
    pub fn print_bold(&mut self, s: &str) -> io::Result<()> {
        queue!(
            self.stdout,
            SetAttribute(Attribute::Bold),
            Print(s),
            SetAttribute(Attribute::Reset),
        )?;
        Ok(())
    }

    /// Set colors.
    pub fn set_colors(&mut self, fg: Color, bg: Color) -> io::Result<()> {
        queue!(
            self.stdout,
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
        )?;
        Ok(())
    }

    /// Reset attributes.
    pub fn reset_attrs(&mut self) -> io::Result<()> {
        queue!(self.stdout, SetAttribute(Attribute::Reset))?;
        Ok(())
    }

    /// Flush all queued output to the terminal.
    pub fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}

/// Translate a crossterm KeyEvent to our KeyCode.
fn translate_key(key: KeyEvent) -> KeyCode {
    use crossterm::event::KeyCode as CK;

    let mods = key.modifiers;
    let has_ctrl = mods.contains(KeyModifiers::CONTROL);
    let has_alt = mods.contains(KeyModifiers::ALT);
    let has_shift = mods.contains(KeyModifiers::SHIFT);

    match key.code {
        CK::Char(c) => {
            if has_ctrl && has_alt {
                KeyCode::AltCtrl(c.to_ascii_lowercase())
            } else if has_ctrl {
                KeyCode::Ctrl(c.to_ascii_lowercase())
            } else if has_alt {
                KeyCode::Alt(c)
            } else {
                KeyCode::Char(c)
            }
        }
        CK::F(n) => KeyCode::F(n),
        CK::Backspace => {
            if has_alt {
                KeyCode::Alt('\x08')
            } else {
                KeyCode::Backspace
            }
        }
        CK::Enter => KeyCode::Enter,
        CK::Left => {
            if has_ctrl && has_shift {
                KeyCode::Special(SpecialKey::ShiftControlLeft)
            } else if has_ctrl {
                KeyCode::Special(SpecialKey::ControlLeft)
            } else if has_alt && has_shift {
                KeyCode::Special(SpecialKey::ShiftAltLeft)
            } else if has_alt {
                KeyCode::Special(SpecialKey::AltLeft)
            } else {
                KeyCode::Left
            }
        }
        CK::Right => {
            if has_ctrl && has_shift {
                KeyCode::Special(SpecialKey::ShiftControlRight)
            } else if has_ctrl {
                KeyCode::Special(SpecialKey::ControlRight)
            } else if has_alt && has_shift {
                KeyCode::Special(SpecialKey::ShiftAltRight)
            } else if has_alt {
                KeyCode::Special(SpecialKey::AltRight)
            } else {
                KeyCode::Right
            }
        }
        CK::Up => {
            if has_ctrl {
                KeyCode::Special(SpecialKey::ControlUp)
            } else if has_alt {
                KeyCode::Special(SpecialKey::AltUp)
            } else {
                KeyCode::Up
            }
        }
        CK::Down => {
            if has_ctrl {
                KeyCode::Special(SpecialKey::ControlDown)
            } else if has_alt {
                KeyCode::Special(SpecialKey::AltDown)
            } else {
                KeyCode::Down
            }
        }
        CK::Home => {
            if has_ctrl {
                KeyCode::Special(SpecialKey::ControlHome)
            } else {
                KeyCode::Home
            }
        }
        CK::End => {
            if has_ctrl {
                KeyCode::Special(SpecialKey::ControlEnd)
            } else {
                KeyCode::End
            }
        }
        CK::PageUp => KeyCode::PageUp,
        CK::PageDown => KeyCode::PageDown,
        CK::Tab => {
            if has_shift {
                KeyCode::Special(SpecialKey::ShiftTab)
            } else {
                KeyCode::Tab
            }
        }
        CK::Delete => {
            if has_ctrl {
                KeyCode::Special(SpecialKey::ControlDelete)
            } else {
                KeyCode::Delete
            }
        }
        CK::Insert => {
            if has_alt {
                KeyCode::Special(SpecialKey::AltInsert)
            } else {
                KeyCode::Insert
            }
        }
        CK::Esc => KeyCode::Escape,
        CK::Null => KeyCode::Null,
        _ => KeyCode::Unknown(0),
    }
}

// ── Screen drawing (impl on Editor) ───────────────────────────────────

impl Editor {
    /// Draw the entire screen.
    pub fn full_refresh(&mut self) -> io::Result<()> {
        self.tio.clear_screen()?;
        self.draw_titlebar()?;
        self.draw_edit_window()?;
        self.draw_statusbar()?;
        if !self.flags.contains(EditorFlags::NO_HELP) {
            self.draw_help_lines()?;
        }
        self.tio.flush()?;
        Ok(())
    }

    /// Draw the title bar.
    fn draw_titlebar(&mut self) -> io::Result<()> {
        self.tio.move_cursor(0, 0)?;
        let buf = &self.buffers[self.current_buf];
        let version = "rano 0.1.0";
        let filename = if buf.filename.is_empty() {
            "New Buffer"
        } else {
            &buf.filename
        };
        let modified = if buf.modified { " [Modified]" } else { "" };

        let title = format!("  {}       {}{}",
            version,
            filename,
            modified,
        );

        // Pad to full width
        let padded = format!("{:<width$}", title, width = self.term_cols);
        self.tio.print_reversed(&padded)?;
        Ok(())
    }

    /// Draw the main editing area.
    fn draw_edit_window(&mut self) -> io::Result<()> {
        let buf = &self.buffers[self.current_buf];
        let show_line_nums = self.flags.contains(EditorFlags::LINE_NUMBERS);

        for row in 0..self.editwinrows {
            let line_index = buf.edittop + row;
            self.tio.move_cursor((row + 1) as u16, 0)?;
            self.tio.clear_to_eol()?;

            if line_index < buf.lines.len() {
                let line = &buf.lines[line_index];

                // Draw line number
                if show_line_nums {
                    let num_str = format!("{:>width$} ", line.lineno, width = self.margin - 1);
                    self.tio.set_colors(Color::DarkYellow, Color::Reset)?;
                    self.tio.print_str(&num_str)?;
                    self.tio.reset_attrs()?;
                }

                // Draw the line content
                let display = self.make_display_string(&line.data, self.editwincols);
                self.tio.print_str(&display)?;
            } else {
                // Past end of file: draw empty or tilde
                if show_line_nums {
                    let blank = format!("{:>width$} ", "", width = self.margin - 1);
                    self.tio.print_str(&blank)?;
                }
            }
        }
        Ok(())
    }

    /// Draw the status bar.
    fn draw_statusbar(&mut self) -> io::Result<()> {
        let row = self.term_rows - if self.flags.contains(EditorFlags::NO_HELP) { 1 } else { 3 };
        self.tio.move_cursor(row as u16, 0)?;

        if !self.statusmsg.is_empty() {
            let padded = format!("{:<width$}", &self.statusmsg, width = self.term_cols);
            if self.lastmessage >= MessageType::Alert {
                self.tio.set_colors(Color::White, Color::Red)?;
                self.tio.print_str(&padded)?;
                self.tio.reset_attrs()?;
            } else {
                self.tio.print_reversed(&padded)?;
            }
            // Clear message after displaying once
            self.statusmsg.clear();
            self.lastmessage = MessageType::Vacuum;
        } else {
            // Blank status bar during normal editing (like nano)
            let padded = format!("{:<width$}", "", width = self.term_cols);
            self.tio.print_str(&padded)?;
        }

        Ok(())
    }

    /// Draw the bottom help lines.
    fn draw_help_lines(&mut self) -> io::Result<()> {
        let help_shortcuts = [
            ("^G", "Help"),
            ("^O", "Write Out"),
            ("^W", "Where Is"),
            ("^K", "Cut"),
            ("^T", "Execute"),
            ("^C", "Location"),
            ("^X", "Exit"),
            ("^R", "Read File"),
            ("^\\", "Replace"),
            ("^U", "Paste"),
            ("^J", "Justify"),
            ("^_", "Go To Line"),
        ];

        let cols_per_entry = self.term_cols / 6;

        // First help row
        let row1 = self.term_rows - 2;
        self.tio.move_cursor(row1 as u16, 0)?;
        self.tio.clear_to_eol()?;
        for i in 0..6.min(help_shortcuts.len()) {
            let (key, tag) = help_shortcuts[i];
            self.tio.print_reversed(key)?;
            let tag_display = if tag.len() > cols_per_entry - key.len() {
                &tag[..cols_per_entry.saturating_sub(key.len() + 1)]
            } else {
                tag
            };
            self.tio.print_str(&format!(" {:<width$}", tag_display, width = cols_per_entry.saturating_sub(key.len() + 1)))?;
        }

        // Second help row
        let row2 = self.term_rows - 1;
        self.tio.move_cursor(row2 as u16, 0)?;
        self.tio.clear_to_eol()?;
        for i in 6..12.min(help_shortcuts.len()) {
            let (key, tag) = help_shortcuts[i];
            self.tio.print_reversed(key)?;
            let tag_display = if tag.len() > cols_per_entry - key.len() {
                &tag[..cols_per_entry.saturating_sub(key.len() + 1)]
            } else {
                tag
            };
            self.tio.print_str(&format!(" {:<width$}", tag_display, width = cols_per_entry.saturating_sub(key.len() + 1)))?;
        }

        Ok(())
    }

    /// Place the cursor at the correct position.
    pub fn place_the_cursor(&mut self) -> io::Result<()> {
        let buf = &self.buffers[self.current_buf];
        let col = chars::wideness(&buf.lines[buf.current].data, buf.current_x, self.tabsize);

        // Adjust edittop if cursor is off screen
        let mut adjusted = false;
        {
            let editwinrows = self.editwinrows;
            let buf = self.current_buffer_mut();
            if buf.current < buf.edittop {
                buf.edittop = buf.current;
                adjusted = true;
            } else if buf.current >= buf.edittop + editwinrows {
                buf.edittop = buf.current.saturating_sub(editwinrows - 1);
                adjusted = true;
            }
        }

        if adjusted {
            self.draw_edit_window()?;
            self.draw_statusbar()?;
            if !self.flags.contains(EditorFlags::NO_HELP) {
                self.draw_help_lines()?;
            }
        }

        let buf = &self.buffers[self.current_buf];
        let screen_row = (buf.current - buf.edittop + 1) as u16;

        let page_start = utils::get_page_start(col, self.editwincols);
        let screen_col = (self.margin + col - page_start) as u16;

        self.tio.move_cursor(screen_row, screen_col)?;
        self.tio.show_cursor()?;
        self.tio.flush()?;
        Ok(())
    }

    /// Create a display string for a line, handling tabs and control characters.
    fn make_display_string(&self, data: &str, max_cols: usize) -> String {
        let mut result = String::new();
        let mut col = 0;

        for c in data.chars() {
            if col >= max_cols {
                break;
            }
            if c == '\t' {
                let spaces = self.tabsize - (col % self.tabsize);
                for _ in 0..spaces.min(max_cols - col) {
                    result.push(' ');
                }
                col += spaces;
            } else if c.is_control() {
                let rep = chars::control_rep(c);
                if col + 2 <= max_cols {
                    result.push('^');
                    result.push(rep);
                    col += 2;
                }
            } else {
                let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
                if col + w <= max_cols {
                    result.push(c);
                    col += w;
                } else {
                    break;
                }
            }
        }

        result
    }
}
