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


/// RC file parsing. Port of nano's rcfile.c.
/// Reads .nanorc / nanorc configuration files and applies settings.

use crate::definitions::*;
use crate::editor::Editor;
use crate::utils;

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "color")]
use crossterm::style::{Attribute, Attributes};

/// The name of the user's RC file.
const HOME_RC_NAME: &str = ".nanorc";

/// System-wide RC file locations to check.
const SYSTEM_RC_PATHS: &[&str] = &[
    "/etc/nanorc",
    "/usr/local/etc/nanorc",
    "/usr/share/nano",
];

/// Directories that may contain .nanorc syntax definition files.
const SYNTAX_DIRS: &[&str] = &[
    "/usr/share/nano",
    "/usr/local/share/nano",
    "/opt/homebrew/share/nano",
];

/// Known set options and their corresponding flags.
const RC_OPTIONS: &[(&str, Option<EditorFlags>)] = &[
    ("boldtext", Some(EditorFlags::BOLD_TEXT)),
    ("casesensitive", Some(EditorFlags::CASE_SENSITIVE)),
    ("constantshow", Some(EditorFlags::CONSTANT_SHOW)),
    ("linenumbers", Some(EditorFlags::LINE_NUMBERS)),
    ("mouse", Some(EditorFlags::USE_MOUSE)),
    ("nohelp", Some(EditorFlags::NO_HELP)),
    ("nonewlines", Some(EditorFlags::NO_NEWLINES)),
    ("nowrap", Some(EditorFlags::NO_WRAP)),
    ("preserve", Some(EditorFlags::PRESERVE)),
    ("autoindent", Some(EditorFlags::AUTOINDENT)),
    ("indicator", Some(EditorFlags::INDICATOR)),
    ("jumpyscrolling", Some(EditorFlags::JUMPY_SCROLLING)),
    ("smarthome", Some(EditorFlags::SMART_HOME)),
    ("softwrap", Some(EditorFlags::SOFTWRAP)),
    ("tabstospaces", Some(EditorFlags::TABS_TO_SPACES)),
    ("trimblanks", Some(EditorFlags::TRIM_BLANKS)),
    ("emptyline", Some(EditorFlags::EMPTY_LINE)),
    ("locking", Some(EditorFlags::LOCKING)),
    ("minibar", Some(EditorFlags::MINIBAR)),
    ("stateflags", Some(EditorFlags::STATEFLAGS)),
    ("zap", Some(EditorFlags::LET_THEM_ZAP)),
    ("zero", Some(EditorFlags::ZERO)),
    ("breaklonglines", Some(EditorFlags::BREAK_LONG_LINES)),
    ("historylog", Some(EditorFlags::HISTORYLOG)),
    ("positionlog", Some(EditorFlags::POSITIONLOG)),
    ("showcursor", Some(EditorFlags::SHOW_CURSOR)),
    ("afterends", Some(EditorFlags::AFTER_ENDS)),
    ("atblanks", Some(EditorFlags::AT_BLANKS)),
    ("bookstyle", Some(EditorFlags::BOOKSTYLE)),
    ("colonparsing", Some(EditorFlags::COLON_PARSING)),
    ("wordbounds", Some(EditorFlags::WORD_BOUNDS)),
    ("rawsequences", Some(EditorFlags::RAW_SEQUENCES)),
    ("modernbindings", Some(EditorFlags::MODERN_BINDINGS)),
    // Options with values (flag is None — handled separately)
    ("tabsize", None),
    ("fill", None),
    ("wordchars", None),
    ("whitespace", None),
    ("matchbrackets", None),
    ("punct", None),
    ("quotestr", None),
];

impl Editor {
    /// Parse and apply RC files (unless --ignorercfiles was given).
    pub fn do_rcfiles(&mut self) {
        // Try the user's home RC file first.
        if let Some(ref home) = self.homedir.clone() {
            let user_rc = PathBuf::from(home).join(HOME_RC_NAME);
            if user_rc.exists() {
                self.parse_rcfile(&user_rc);
            }
        }

        // Fall back to system RC files if user RC didn't exist.
        #[cfg(feature = "color")]
        if self.syntaxes.is_empty() {
            for path_str in SYSTEM_RC_PATHS {
                let path = Path::new(path_str);
                if path.is_file() {
                    self.parse_rcfile(path);
                    break;
                }
            }
        }

        // Load syntax definitions from system directories if none loaded yet.
        #[cfg(feature = "color")]
        if self.syntaxes.is_empty() {
            self.load_syntax_dirs();
        }
    }

    /// Load all .nanorc syntax files from known system directories.
    #[cfg(feature = "color")]
    fn load_syntax_dirs(&mut self) {
        for dir_str in SYNTAX_DIRS {
            let dir = Path::new(dir_str);
            if dir.is_dir() {
                if let Ok(entries) = fs::read_dir(dir) {
                    let mut paths: Vec<PathBuf> = entries
                        .flatten()
                        .filter(|e| {
                            e.path().extension().map(|ext| ext == "nanorc").unwrap_or(false)
                        })
                        .map(|e| e.path())
                        .collect();
                    paths.sort();
                    for path in paths {
                        self.parse_rcfile(&path);
                    }
                }
                if !self.syntaxes.is_empty() {
                    return;
                }
            }
        }
    }

    /// Parse a single RC file and apply its directives.
    fn parse_rcfile(&mut self, path: &Path) {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip comments and empty lines.
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
            let command = parts[0];
            let argument = parts.get(1).map(|s| s.trim()).unwrap_or("");

            match command {
                "set" => self.parse_set_option(argument),
                "unset" => self.parse_unset_option(argument),
                "include" => self.parse_include(argument),
                #[cfg(feature = "color")]
                "syntax" => self.parse_syntax_directive(argument),
                #[cfg(feature = "color")]
                "color" | "icolor" => self.parse_color_directive(argument, command == "icolor"),
                #[cfg(feature = "color")]
                "header" => self.parse_header_directive(argument),
                #[cfg(feature = "color")]
                "magic" => { /* Magic match — not commonly used */ }
                #[cfg(feature = "color")]
                "comment" => self.parse_comment_directive(argument),
                #[cfg(feature = "color")]
                "linter" => {
                    if let Some(syn) = self.syntaxes.last_mut() {
                        syn.linter = Some(argument.trim_matches('"').to_string());
                    }
                }
                #[cfg(feature = "color")]
                "formatter" => {
                    if let Some(syn) = self.syntaxes.last_mut() {
                        syn.formatter = Some(argument.trim_matches('"').to_string());
                    }
                }
                #[cfg(feature = "color")]
                "tabgives" => {
                    if let Some(syn) = self.syntaxes.last_mut() {
                        syn.tabstring = Some(argument.trim_matches('"').to_string());
                    }
                }
                "bind" | "unbind" => { /* Key binding changes — TODO */ }
                "extendsyntax" => { /* Extend syntax — TODO */ }
                _ => {
                    // Unknown directive; silently ignore.
                }
            }
        }
    }

    /// Handle a "set option [value]" directive.
    fn parse_set_option(&mut self, argument: &str) {
        let parts: Vec<&str> = argument.splitn(2, char::is_whitespace).collect();
        let option_name = parts[0];
        let value = parts.get(1).map(|s| s.trim()).unwrap_or("");

        // Check for flag-based options.
        for &(name, flag_opt) in RC_OPTIONS {
            if name == option_name {
                if let Some(flag) = flag_opt {
                    self.flags |= flag;
                    return;
                }
                // Value-based options.
                match option_name {
                    "tabsize" => {
                        if let Ok(n) = value.parse::<usize>() {
                            if n > 0 && n <= 8192 {
                                self.tabsize = n;
                            }
                        }
                    }
                    "wordchars" => {
                        let unquoted = value.trim_matches('"');
                        self.word_chars = Some(unquoted.to_string());
                    }
                    _ => {}
                }
                return;
            }
        }
    }

    /// Handle an "unset option" directive.
    fn parse_unset_option(&mut self, argument: &str) {
        let option_name = argument.trim();
        for &(name, flag_opt) in RC_OPTIONS {
            if name == option_name {
                if let Some(flag) = flag_opt {
                    self.flags.remove(flag);
                }
                return;
            }
        }
    }

    /// Parse a "syntax name "ext..."" directive — starts a new syntax definition.
    #[cfg(feature = "color")]
    fn parse_syntax_directive(&mut self, argument: &str) {
        // Format: syntax name "ext1" "ext2" ...
        // or:     syntax "name" "ext1" "ext2" ...
        let mut rest = argument;

        // Extract name (may be quoted)
        let name = if rest.starts_with('"') {
            let end = rest[1..].find('"').map(|i| i + 1).unwrap_or(rest.len());
            let n = &rest[1..end];
            rest = rest[end + 1..].trim();
            n.to_string()
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            let n = &rest[..end];
            rest = rest[end..].trim();
            n.to_string()
        };

        // Parse extension regexes
        let mut extensions = Vec::new();
        for token in extract_quoted_strings(rest) {
            if let Ok(re) = regex::Regex::new(&token) {
                extensions.push(RegexEntry { regex: re });
            }
        }

        let syn = SyntaxDef {
            name,
            filename: None,
            lineno: 0,
            extensions,
            headers: Vec::new(),
            magics: Vec::new(),
            linter: None,
            formatter: None,
            tabstring: None,
            #[cfg(feature = "comment")]
            comment: None,
            colors: Vec::new(),
            multiscore: 0,
        };
        self.syntaxes.push(syn);
    }

    /// Parse a "color fg[,bg] "regex"" or "color fg[,bg] start="regex" end="regex"" directive.
    #[cfg(feature = "color")]
    fn parse_color_directive(&mut self, argument: &str, case_insensitive: bool) {
        if self.syntaxes.is_empty() {
            return;
        }

        // Format: color fg[,bg] "regex" ["regex"...]
        //     or: color fg[,bg] start="regex" end="regex"
        let parts: Vec<&str> = argument.splitn(2, char::is_whitespace).collect();
        if parts.len() < 2 {
            return;
        }

        let color_spec = parts[0];
        let pattern_part = parts[1].trim();

        // Parse color spec: "fg" or "fg,bg" with optional "bright" or "bold," prefix
        let (fg, bg, attrs) = parse_color_spec(color_spec);

        let rule_id = self.syntaxes.last().map(|s| s.colors.len() as i16).unwrap_or(0);

        // Check for start="..." end="..." (multi-line pattern)
        if pattern_part.starts_with("start=") {
            let (start_re, end_re) = parse_start_end_pattern(pattern_part, case_insensitive);
            if let Some(start) = start_re {
                let rule = ColorRule {
                    id: rule_id,
                    fg,
                    bg,
                    attributes: attrs,
                    start,
                    end: end_re,
                };
                if let Some(syn) = self.syntaxes.last_mut() {
                    syn.colors.push(rule);
                }
            }
        } else {
            // Single-line patterns: one or more quoted regexes
            for token in extract_quoted_strings(pattern_part) {
                let pattern = if case_insensitive {
                    format!("(?i){}", token)
                } else {
                    token
                };
                if let Ok(re) = regex::Regex::new(&pattern) {
                    let rule = ColorRule {
                        id: rule_id,
                        fg,
                        bg,
                        attributes: attrs,
                        start: re,
                        end: None,
                    };
                    if let Some(syn) = self.syntaxes.last_mut() {
                        syn.colors.push(rule);
                    }
                }
            }
        }
    }

    /// Parse a "header "regex"" directive.
    #[cfg(feature = "color")]
    fn parse_header_directive(&mut self, argument: &str) {
        if self.syntaxes.is_empty() {
            return;
        }
        for token in extract_quoted_strings(argument) {
            if let Ok(re) = regex::Regex::new(&token) {
                if let Some(syn) = self.syntaxes.last_mut() {
                    syn.headers.push(RegexEntry { regex: re });
                }
            }
        }
    }

    /// Parse a "comment "str"" directive.
    #[cfg(feature = "color")]
    fn parse_comment_directive(&mut self, argument: &str) {
        let s = argument.trim().trim_matches('"');
        if let Some(syn) = self.syntaxes.last_mut() {
            #[cfg(feature = "comment")]
            {
                syn.comment = Some(s.to_string());
            }
        }
    }

    /// Handle an "include path" directive (for including syntax files).
    fn parse_include(&mut self, argument: &str) {
        let path_str = argument.trim().trim_matches('"');
        let expanded = utils::real_dir_from_tilde(path_str);

        // Handle glob patterns.
        if expanded.contains('*') || expanded.contains('?') {
            if let Ok(entries) = glob_paths(&expanded) {
                for entry in entries {
                    self.parse_rcfile(&entry);
                }
            }
        } else {
            let path = Path::new(&expanded);
            if path.exists() {
                self.parse_rcfile(path);
            }
        }
    }
}

/// Extract quoted strings from a directive argument.
/// e.g., `"foo" "bar"` -> vec!["foo", "bar"]
fn extract_quoted_strings(input: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut rest = input;
    while let Some(start) = rest.find('"') {
        rest = &rest[start + 1..];
        if let Some(end) = rest.find('"') {
            results.push(rest[..end].to_string());
            rest = &rest[end + 1..];
        } else {
            // Unterminated quote — take the rest
            results.push(rest.to_string());
            break;
        }
    }
    results
}

/// Parse a color name to its numeric value.
/// Supports: black, red, green, yellow, blue, magenta, cyan, white,
/// normal (-1), and "bright" prefix for bold/bright variants.
#[cfg(feature = "color")]
fn parse_color_name(name: &str) -> (i16, bool) {
    let name = name.trim();
    let (is_bright, base) = if let Some(rest) = name.strip_prefix("bright") {
        (true, rest)
    } else if let Some(rest) = name.strip_prefix("bold") {
        (true, rest)
    } else {
        (false, name)
    };

    let val = match base {
        "black" => 0,
        "red" => 1,
        "green" => 2,
        "yellow" => 3,
        "blue" => 4,
        "magenta" => 5,
        "cyan" => 6,
        "white" => 7,
        "normal" | "" => -1,
        _ => -1,
    };

    // For bright colors, add 8 to get the bright ANSI variant
    let val = if is_bright && val >= 0 { val + 8 } else { val };
    (val, is_bright)
}

/// Parse a color spec like "green" or "brightred,blue" or "bold,yellow".
/// Returns (fg, bg, attributes).
#[cfg(feature = "color")]
fn parse_color_spec(spec: &str) -> (i16, i16, Attributes) {
    let mut attrs = Attributes::default();
    let parts: Vec<&str> = spec.split(',').collect();

    let (fg, is_bright) = parse_color_name(parts[0]);
    if is_bright {
        attrs.set(Attribute::Bold);
    }

    let bg = if parts.len() > 1 {
        let (bg_val, _) = parse_color_name(parts[1]);
        bg_val
    } else {
        -1
    };

    (fg, bg, attrs)
}

/// Parse start="regex" end="regex" patterns.
#[cfg(feature = "color")]
fn parse_start_end_pattern(input: &str, case_insensitive: bool) -> (Option<regex::Regex>, Option<regex::Regex>) {
    let prefix = if case_insensitive { "(?i)" } else { "" };

    let mut start_re = None;
    let mut end_re = None;

    // Find start="..."
    if let Some(pos) = input.find("start=\"") {
        let rest = &input[pos + 7..];
        if let Some(end_pos) = rest.find('"') {
            let pattern = format!("{}{}", prefix, &rest[..end_pos]);
            start_re = regex::Regex::new(&pattern).ok();
        }
    }

    // Find end="..."
    if let Some(pos) = input.find("end=\"") {
        let rest = &input[pos + 5..];
        if let Some(end_pos) = rest.find('"') {
            let pattern = format!("{}{}", prefix, &rest[..end_pos]);
            end_re = regex::Regex::new(&pattern).ok();
        }
    }

    (start_re, end_re)
}

/// Simple glob expansion for include directives.
fn glob_paths(pattern: &str) -> Result<Vec<PathBuf>, ()> {
    let mut results = Vec::new();

    // Use std::fs to manually expand — we keep it simple.
    // Split the pattern into directory part and file glob.
    let path = Path::new(pattern);
    let parent = path.parent().unwrap_or(Path::new("."));
    let file_pattern = path.file_name().map(|f| f.to_string_lossy().to_string());

    if let Some(pat) = file_pattern {
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if simple_glob_match(&pat, &name) {
                    results.push(entry.path());
                }
            }
        }
    }

    results.sort();
    Ok(results)
}

/// Very simple glob matching (supports * and ?).
fn simple_glob_match(pattern: &str, text: &str) -> bool {
    fn match_recursive(
        p: &[char],
        t: &[char],
    ) -> bool {
        if p.is_empty() {
            return t.is_empty();
        }
        if p[0] == '*' {
            // Try matching zero or more characters.
            for i in 0..=t.len() {
                if match_recursive(&p[1..], &t[i..]) {
                    return true;
                }
            }
            return false;
        }
        if t.is_empty() {
            return false;
        }
        if p[0] == '?' || p[0] == t[0] {
            return match_recursive(&p[1..], &t[1..]);
        }
        false
    }

    let pc: Vec<char> = pattern.chars().collect();
    let tc: Vec<char> = text.chars().collect();
    match_recursive(&pc, &tc)
}
