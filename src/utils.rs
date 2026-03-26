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


/// General utility functions. Port of nano's utils.c.

use std::path::Path;

/// Return the filename part of the given path.
pub fn tail(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
}

/// Return the number of decimal digits needed to represent the given number.
pub fn digits(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    ((n as f64).log10().floor() as usize) + 1
}

/// Parse a number from a string.
pub fn parse_num(s: &str) -> Option<isize> {
    s.trim().parse().ok()
}

/// Parse "line,column" or "line:column" or "line.column" from a string.
pub fn parse_line_column(s: &str) -> (Option<isize>, Option<isize>) {
    let s = s.trim();
    if let Some(sep_pos) = s.find(|c: char| c == ',' || c == '.' || c == ':') {
        let line_part = &s[..sep_pos];
        let col_part = &s[sep_pos + 1..];
        let line = if line_part.is_empty() {
            None
        } else {
            parse_num(line_part)
        };
        let col = if col_part.is_empty() {
            None
        } else {
            parse_num(col_part)
        };
        (line, col)
    } else {
        (parse_num(s), None)
    }
}

/// Get the user's home directory.
pub fn get_homedir() -> Option<String> {
    dirs::home_dir().map(|p| p.to_string_lossy().into_owned())
}

/// Expand a path that starts with ~/ to the user's home directory.
pub fn real_dir_from_tilde(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = get_homedir() {
            if path == "~" {
                return home;
            }
            return format!("{}{}", home, &path[1..]);
        }
    }
    path.to_string()
}

/// Return the column number of the first character displayed in the edit
/// window when the cursor is at the given column (horizontal scrolling).
pub fn get_page_start(column: usize, editwincols: usize) -> usize {
    if column == 0 || column + 2 < editwincols {
        0
    } else if editwincols > 8 {
        column - 6 - (column - 6) % (editwincols - 8)
    } else {
        column.saturating_sub(editwincols - 2)
    }
}

/// Check if a word at the given position and length is a separate word.
pub fn is_separate_word(text: &str, position: usize, length: usize) -> bool {
    let before_ok = position == 0
        || text[..position]
            .chars()
            .next_back()
            .map_or(true, |c| !c.is_alphabetic());
    let after_ok = position + length >= text.len()
        || text[position + length..]
            .chars()
            .next()
            .map_or(true, |c| !c.is_alphabetic());
    before_ok && after_ok
}
