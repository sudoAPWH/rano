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


/// RC file parsing. Port of nano's rcfile.c.
/// Reads .nanorc / nanorc configuration files and applies settings.

use crate::definitions::*;
use crate::editor::Editor;
use crate::utils;

use std::fs;
use std::path::{Path, PathBuf};

/// The name of the user's RC file.
const HOME_RC_NAME: &str = ".nanorc";

/// System-wide RC file locations to check.
const SYSTEM_RC_PATHS: &[&str] = &[
    "/etc/nanorc",
    "/usr/local/etc/nanorc",
    "/usr/share/nano",
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
        if let Some(ref home) = self.homedir {
            let user_rc = PathBuf::from(home).join(HOME_RC_NAME);
            if user_rc.exists() {
                self.parse_rcfile(&user_rc);
                return;
            }
        }

        // Fall back to system RC files.
        for path_str in SYSTEM_RC_PATHS {
            let path = Path::new(path_str);
            if path.is_file() {
                self.parse_rcfile(path);
                return;
            }
            // Also check for nanorc file within a directory of .nanorc files
            if path.is_dir() {
                // Directories like /usr/share/nano contain syntax files, not the main rc
                continue;
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
                "syntax" => { /* Syntax definition parsing — TODO for full color support */ }
                #[cfg(feature = "color")]
                "color" | "icolor" => { /* Color rule parsing — TODO */ }
                #[cfg(feature = "color")]
                "header" => { /* Header match — TODO */ }
                #[cfg(feature = "color")]
                "magic" => { /* Magic match — TODO */ }
                #[cfg(feature = "color")]
                "comment" => { /* Comment string — TODO */ }
                #[cfg(feature = "color")]
                "linter" => { /* Linter command — TODO */ }
                #[cfg(feature = "color")]
                "formatter" => { /* Formatter command — TODO */ }
                #[cfg(feature = "color")]
                "tabgives" => { /* Tab replacement — TODO */ }
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
