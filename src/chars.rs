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


/// Character classification and manipulation utilities.
/// Port of nano's chars.c — in Rust, most of this is handled natively
/// by char/str methods, but we keep the API for compatibility.

use unicode_width::UnicodeWidthChar;

/// Return true when the given character is alphanumeric.
pub fn is_alnum_char(c: char) -> bool {
    c.is_alphanumeric()
}

/// Return true when the given character is alphabetic.
pub fn is_alpha_char(c: char) -> bool {
    c.is_alphabetic()
}

/// Return true when the given character is a blank (space or tab).
pub fn is_blank_char(c: char) -> bool {
    c == ' ' || c == '\t'
}

/// Return true when the given character is a control character.
pub fn is_cntrl_char(c: char) -> bool {
    c.is_control()
}

/// Return true when the given character is punctuation.
pub fn is_punct_char(c: char) -> bool {
    c.is_ascii_punctuation() || (!c.is_alphanumeric() && !c.is_whitespace() && !c.is_control())
}

/// Return true when the given character is word-forming.
/// A word character is alphanumeric, or in word_chars, or punctuation if allowed.
pub fn is_word_char(c: char, allow_punct: bool, word_chars: &Option<String>) -> bool {
    if c == '\0' {
        return false;
    }
    if is_alnum_char(c) {
        return true;
    }
    if allow_punct && is_punct_char(c) {
        return true;
    }
    if let Some(ref wc) = word_chars {
        if !wc.is_empty() {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            return wc.contains(&*s);
        }
    }
    false
}

/// Return the visible representation of a control character.
pub fn control_rep(c: char) -> char {
    let byte = c as u8;
    if byte == 0x7F {
        '?'
    } else if byte < 0x20 {
        (byte + 64) as char
    } else {
        c
    }
}

/// Return the visible representation of a control character in data context.
pub fn control_mbrep(c: char, is_data: bool) -> char {
    if c == '\n' && is_data {
        '@'
    } else if c.is_control() {
        control_rep(c)
    } else {
        c
    }
}

/// Return true when the given character occupies two cells on screen.
pub fn is_doublewidth(c: char) -> bool {
    UnicodeWidthChar::width(c).unwrap_or(1) == 2
}

/// Return true when the given character occupies zero cells on screen.
pub fn is_zerowidth(c: char) -> bool {
    UnicodeWidthChar::width(c).unwrap_or(1) == 0
}

/// Return the display width of a character, accounting for tabs.
pub fn char_width(c: char, column: usize, tabsize: usize) -> usize {
    if c == '\t' {
        tabsize - (column % tabsize)
    } else if c.is_control() {
        2 // control chars display as ^X
    } else {
        UnicodeWidthChar::width(c).unwrap_or(1)
    }
}

/// Advance over a character in a string, returning its display width.
/// Updates column to reflect the new position.
pub fn advance_over(c: char, column: &mut usize, tabsize: usize) -> usize {
    let w = char_width(c, *column, tabsize);
    *column += w;
    w
}

/// Return the display width of a string up to max_bytes bytes.
pub fn wideness(text: &str, max_bytes: usize, tabsize: usize) -> usize {
    let mut width = 0;
    let mut bytes = 0;
    for c in text.chars() {
        let clen = c.len_utf8();
        if bytes + clen > max_bytes {
            break;
        }
        advance_over(c, &mut width, tabsize);
        bytes += clen;
    }
    width
}

/// Return the total display width of a string.
pub fn breadth(text: &str, tabsize: usize) -> usize {
    let mut width = 0;
    for c in text.chars() {
        advance_over(c, &mut width, tabsize);
    }
    width
}

/// Return the byte index in text of the character that will not overshoot
/// the given column.
pub fn actual_x(text: &str, column: usize, tabsize: usize) -> usize {
    let mut width = 0;
    let mut byte_pos = 0;
    for c in text.chars() {
        let old_width = width;
        advance_over(c, &mut width, tabsize);
        if width > column {
            // If we haven't moved at all, include at least one char
            if old_width == 0 && byte_pos == 0 {
                return c.len_utf8();
            }
            return byte_pos;
        }
        byte_pos += c.len_utf8();
    }
    byte_pos
}

/// Return true when the string is empty or consists only of blanks.
pub fn white_string(s: &str) -> bool {
    s.chars().all(|c| is_blank_char(c) || c == '\r')
}

/// Strip leading blanks from a string.
pub fn strip_leading_blanks(s: &str) -> &str {
    s.trim_start_matches(|c: char| c == ' ' || c == '\t')
}

/// Case-insensitive search for needle in haystack.
pub fn strcasestr(haystack: &str, needle: &str) -> Option<usize> {
    let haystack_lower = haystack.to_lowercase();
    let needle_lower = needle.to_lowercase();
    haystack_lower.find(&needle_lower)
}

/// Reverse search for needle in haystack, starting from a byte position.
pub fn revstrstr(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    haystack[..from.min(haystack.len())].rfind(needle)
}

/// Reverse case-insensitive search.
pub fn rev_strcasestr(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    let end = from.min(haystack.len());
    let haystack_lower = haystack[..end].to_lowercase();
    let needle_lower = needle.to_lowercase();
    haystack_lower.rfind(&needle_lower)
}

/// Step left by one character in the string, returning the new byte position.
pub fn step_left(text: &str, pos: usize) -> usize {
    if pos == 0 {
        return 0;
    }
    let mut new_pos = pos - 1;
    while new_pos > 0 && !text.is_char_boundary(new_pos) {
        new_pos -= 1;
    }
    new_pos
}

/// Step right by one character in the string, returning the new byte position.
pub fn step_right(text: &str, pos: usize) -> usize {
    if pos >= text.len() {
        return text.len();
    }
    let mut new_pos = pos + 1;
    while new_pos < text.len() && !text.is_char_boundary(new_pos) {
        new_pos += 1;
    }
    new_pos
}

/// Return the length of the indentation on the given line.
pub fn indent_length(line: &str) -> usize {
    let trimmed = line.trim_start_matches(|c: char| c == ' ' || c == '\t');
    line.len() - trimmed.len()
}
