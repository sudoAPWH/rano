/// File browser. Port of nano's browser.c.
/// Only compiled when the "browser" feature is enabled.

#[cfg(feature = "browser")]
use crate::definitions::*;
#[cfg(feature = "browser")]
use crate::editor::Editor;

#[cfg(feature = "browser")]
use std::fs;
#[cfg(feature = "browser")]
use std::io;
#[cfg(feature = "browser")]
use std::path::{Path, PathBuf};

#[cfg(feature = "browser")]
impl Editor {
    /// Open the file browser to select a file.
    /// Returns the selected file path, or None if cancelled.
    pub fn do_browser(&mut self, starting_dir: &str) -> io::Result<Option<String>> {
        let mut current_dir = PathBuf::from(starting_dir);
        if !current_dir.is_dir() {
            current_dir = current_dir
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
        }

        loop {
            let entries = self.list_directory(&current_dir)?;
            let mut selected = 0usize;
            let mut top = 0usize;

            loop {
                self.draw_browser(&current_dir, &entries, selected, top)?;

                let keycode = self.tio.get_kbinput()?;

                match keycode {
                    KeyCode::Escape | KeyCode::Ctrl('c') => {
                        self.refresh_needed = true;
                        return Ok(None);
                    }
                    KeyCode::Enter => {
                        if selected < entries.len() {
                            let entry = &entries[selected];
                            if entry.is_dir {
                                current_dir = PathBuf::from(&entry.path);
                                break; // Re-list
                            } else {
                                self.refresh_needed = true;
                                return Ok(Some(entry.path.clone()));
                            }
                        }
                    }
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                            if selected < top {
                                top = selected;
                            }
                        }
                    }
                    KeyCode::Down => {
                        if selected + 1 < entries.len() {
                            selected += 1;
                            if selected >= top + self.editwinrows {
                                top = selected - self.editwinrows + 1;
                            }
                        }
                    }
                    KeyCode::PageUp => {
                        selected = selected.saturating_sub(self.editwinrows);
                        top = top.saturating_sub(self.editwinrows);
                    }
                    KeyCode::PageDown => {
                        selected = (selected + self.editwinrows).min(entries.len().saturating_sub(1));
                        if selected >= top + self.editwinrows {
                            top = selected.saturating_sub(self.editwinrows - 1);
                        }
                    }
                    KeyCode::Home => {
                        selected = 0;
                        top = 0;
                    }
                    KeyCode::End => {
                        selected = entries.len().saturating_sub(1);
                        top = selected.saturating_sub(self.editwinrows - 1);
                    }
                    _ => {}
                }
            }
        }
    }

    /// List files and directories in the given path.
    fn list_directory(&self, dir: &Path) -> io::Result<Vec<BrowserEntry>> {
        let mut entries = Vec::new();

        // Add parent directory entry.
        if let Some(parent) = dir.parent() {
            entries.push(BrowserEntry {
                name: "..".to_string(),
                path: parent.to_string_lossy().to_string(),
                is_dir: true,
                size: 0,
            });
        }

        let mut dir_entries: Vec<BrowserEntry> = Vec::new();
        let mut file_entries: Vec<BrowserEntry> = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files.
            if name.starts_with('.') {
                continue;
            }

            let be = BrowserEntry {
                name,
                path: entry.path().to_string_lossy().to_string(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
            };

            if be.is_dir {
                dir_entries.push(be);
            } else {
                file_entries.push(be);
            }
        }

        // Sort directories and files separately, then combine.
        dir_entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        file_entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        entries.extend(dir_entries);
        entries.extend(file_entries);

        Ok(entries)
    }

    /// Draw the file browser screen.
    fn draw_browser(
        &mut self,
        dir: &Path,
        entries: &[BrowserEntry],
        selected: usize,
        top: usize,
    ) -> io::Result<()> {
        // Title bar showing current directory.
        let title = format!(" File Browser: {}", dir.display());
        let padded = format!("{:<width$}", title, width = self.term_cols);
        self.tio.move_cursor(0, 0)?;
        self.tio.print_reversed(&padded)?;

        // List entries.
        for row in 0..self.editwinrows {
            self.tio.move_cursor((row + 1) as u16, 0)?;
            self.tio.clear_to_eol()?;

            let idx = top + row;
            if idx < entries.len() {
                let entry = &entries[idx];
                let prefix = if entry.is_dir { "Dir  " } else { "     " };
                let display = format!("{}{}", prefix, entry.name);
                let truncated = if display.len() > self.term_cols {
                    &display[..self.term_cols]
                } else {
                    &display
                };

                if idx == selected {
                    self.tio.print_reversed(truncated)?;
                    // Pad the rest of the line in reverse.
                    if truncated.len() < self.term_cols {
                        let pad = " ".repeat(self.term_cols - truncated.len());
                        self.tio.print_reversed(&pad)?;
                    }
                } else {
                    self.tio.print_str(truncated)?;
                }
            }
        }

        // Status bar.
        let status_row = self.term_rows
            - if self.flags.contains(EditorFlags::NO_HELP) {
                1
            } else {
                3
            };
        self.tio.move_cursor(status_row as u16, 0)?;
        let status = if selected < entries.len() {
            let e = &entries[selected];
            if e.is_dir {
                format!("Directory: {}", e.name)
            } else {
                format!("{} ({} bytes)", e.name, e.size)
            }
        } else {
            String::new()
        };
        let padded = format!("{:<width$}", status, width = self.term_cols);
        self.tio.print_reversed(&padded)?;

        self.tio.flush()?;
        Ok(())
    }
}

/// A file browser directory entry.
#[cfg(feature = "browser")]
struct BrowserEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
}
