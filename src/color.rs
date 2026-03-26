/// Syntax highlighting and color support. Port of nano's color.c.

#[cfg(feature = "color")]
use crate::definitions::*;
#[cfg(feature = "color")]
use crate::editor::Editor;

#[cfg(feature = "color")]
use crossterm::style::Color;

#[cfg(feature = "color")]
impl Editor {
    /// Initialize the interface color pairs based on loaded syntax definitions.
    pub fn set_interface_colorpairs(&mut self) {
        // Default interface colors using crossterm's Color enum.
        // Unlike ncurses, we don't need to init color pairs — crossterm
        // applies fg/bg directly when rendering.
        self.have_palette = true;
    }

    /// Determine the syntax to use for the current buffer, based on filename
    /// extension or header line matching.
    pub fn find_and_apply_syntax(&mut self) {
        let buf = &self.buffers[self.current_buf];
        let filename = buf.filename.clone();

        if self.flags.contains(EditorFlags::NO_SYNTAX) || self.syntaxes.is_empty() {
            return;
        }

        let mut chosen: Option<usize> = None;

        // First try to match by file extension.
        for (idx, syn) in self.syntaxes.iter().enumerate() {
            for ext in &syn.extensions {
                if ext.regex.is_match(&filename) {
                    chosen = Some(idx);
                    break;
                }
            }
            if chosen.is_some() {
                break;
            }
        }

        // If no extension match, try the first line (header match).
        if chosen.is_none() && !buf.lines.is_empty() {
            let first_line = &buf.lines[0].data;
            for (idx, syn) in self.syntaxes.iter().enumerate() {
                for hdr in &syn.headers {
                    if hdr.regex.is_match(first_line) {
                        chosen = Some(idx);
                        break;
                    }
                }
                if chosen.is_some() {
                    break;
                }
            }
        }

        // Fall back to "default" syntax if one exists.
        if chosen.is_none() {
            for (idx, syn) in self.syntaxes.iter().enumerate() {
                if syn.name == "default" {
                    chosen = Some(idx);
                    break;
                }
            }
        }

        self.buffers[self.current_buf].syntax = chosen;
    }

    /// Determine the coloring for a given line, returning a vector of
    /// (start_byte, end_byte, color_rule_index) spans.
    pub fn color_line(&self, line_data: &str, syntax_idx: usize) -> Vec<(usize, usize, usize)> {
        let mut spans = Vec::new();
        let syntax = &self.syntaxes[syntax_idx];

        for (rule_idx, rule) in syntax.colors.iter().enumerate() {
            if rule.end.is_none() {
                // Single-line regex: find all matches.
                for m in rule.start.find_iter(line_data) {
                    spans.push((m.start(), m.end(), rule_idx));
                }
            }
            // Multi-line (start..end) patterns would need cross-line state tracking.
            // This is a simplified implementation; full multidata tracking is TODO.
        }

        // Sort by start position; later rules override earlier ones.
        spans.sort_by_key(|s| s.0);
        spans
    }

    /// Convert a color rule's fg/bg into crossterm Color values.
    pub fn resolve_color(color_val: i16) -> Color {
        match color_val {
            0 => Color::Black,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            7 => Color::White,
            8..=15 => Color::AnsiValue(color_val as u8),
            -1 => Color::Reset, // default color
            _ => Color::AnsiValue(color_val as u8),
        }
    }
}
