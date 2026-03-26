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


/// Search and position history. Port of nano's history.c.

#[cfg(feature = "histories")]
use crate::definitions::*;
#[cfg(feature = "histories")]
use crate::editor::Editor;
#[cfg(feature = "histories")]
use std::fs;
#[cfg(feature = "histories")]
use std::io::Write as IoWrite;
#[cfg(feature = "histories")]
use std::path::PathBuf;

#[cfg(feature = "histories")]
const SEARCH_HISTORY_FILE: &str = "search_history";
#[cfg(feature = "histories")]
const POSITION_HISTORY_FILE: &str = "filepos_history";
#[cfg(feature = "histories")]
const MAX_HISTORY_ENTRIES: usize = 100;

#[cfg(feature = "histories")]
impl Editor {
    /// Load search and replace history from disk.
    pub fn load_history(&mut self) {
        let history_dir = match self.history_dir() {
            Some(d) => d,
            None => return,
        };

        let search_path = history_dir.join(SEARCH_HISTORY_FILE);
        if let Ok(content) = fs::read_to_string(&search_path) {
            let mut search_items = Vec::new();
            let mut replace_items = Vec::new();
            let mut in_replace = false;

            for line in content.lines() {
                if line.is_empty() {
                    // Empty line separates search from replace history.
                    in_replace = true;
                    continue;
                }
                if in_replace {
                    replace_items.push(line.to_string());
                } else {
                    search_items.push(line.to_string());
                }
            }

            self.search_history = search_items;
            self.replace_history = replace_items;
        }
    }

    /// Save search and replace history to disk.
    pub fn save_history(&self) {
        let history_dir = match self.history_dir() {
            Some(d) => d,
            None => return,
        };

        // Ensure directory exists.
        let _ = fs::create_dir_all(&history_dir);

        let search_path = history_dir.join(SEARCH_HISTORY_FILE);
        if let Ok(mut file) = fs::File::create(&search_path) {
            // Write search history (last MAX_HISTORY_ENTRIES entries).
            let start = self.search_history.len().saturating_sub(MAX_HISTORY_ENTRIES);
            for item in &self.search_history[start..] {
                let _ = writeln!(file, "{}", item);
            }

            // Separator.
            let _ = writeln!(file);

            // Write replace history.
            let start = self.replace_history.len().saturating_sub(MAX_HISTORY_ENTRIES);
            for item in &self.replace_history[start..] {
                let _ = writeln!(file, "{}", item);
            }
        }
    }

    /// Load the position history (file positions) from disk.
    pub fn load_positionlog(&self) -> Vec<PositionEntry> {
        let history_dir = match self.history_dir() {
            Some(d) => d,
            None => return Vec::new(),
        };

        let pos_path = history_dir.join(POSITION_HISTORY_FILE);
        let mut entries = Vec::new();

        if let Ok(content) = fs::read_to_string(&pos_path) {
            for line in content.lines() {
                let parts: Vec<&str> = line.splitn(4, ' ').collect();
                if parts.len() >= 3 {
                    let filename = parts[0].to_string();
                    let linenumber = parts[1].parse::<usize>().unwrap_or(1);
                    let columnnumber = parts[2].parse::<usize>().unwrap_or(1);
                    let anchors = parts.get(3).unwrap_or(&"").to_string();
                    entries.push(PositionEntry {
                        filename,
                        linenumber,
                        columnnumber,
                        anchors,
                    });
                }
            }
        }

        entries
    }

    /// Save the position history to disk.
    pub fn save_positionlog(&self, entries: &[PositionEntry]) {
        let history_dir = match self.history_dir() {
            Some(d) => d,
            None => return,
        };

        let _ = fs::create_dir_all(&history_dir);
        let pos_path = history_dir.join(POSITION_HISTORY_FILE);

        if let Ok(mut file) = fs::File::create(&pos_path) {
            // Keep only the last 200 entries.
            let start = entries.len().saturating_sub(200);
            for entry in &entries[start..] {
                let _ = writeln!(
                    file,
                    "{} {} {} {}",
                    entry.filename, entry.linenumber, entry.columnnumber, entry.anchors
                );
            }
        }
    }

    /// Add a search string to the search history.
    pub fn add_to_search_history(&mut self, item: &str) {
        if item.is_empty() {
            return;
        }
        // Remove duplicates.
        self.search_history.retain(|s| s != item);
        self.search_history.push(item.to_string());
        // Cap at max.
        if self.search_history.len() > MAX_HISTORY_ENTRIES {
            self.search_history.remove(0);
        }
    }

    /// Add a replace string to the replace history.
    pub fn add_to_replace_history(&mut self, item: &str) {
        if item.is_empty() {
            return;
        }
        self.replace_history.retain(|s| s != item);
        self.replace_history.push(item.to_string());
        if self.replace_history.len() > MAX_HISTORY_ENTRIES {
            self.replace_history.remove(0);
        }
    }

    /// Get the directory where history files are stored.
    fn history_dir(&self) -> Option<PathBuf> {
        self.homedir
            .as_ref()
            .map(|h| PathBuf::from(h).join(".local").join("share").join("nano"))
    }
}
