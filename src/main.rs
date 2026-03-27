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


#![allow(dead_code, unused_imports, unused_variables)]

mod definitions;
mod chars;
mod color;
mod cut;
mod editor;
mod files;
mod global;
mod help;
mod history;
mod movement;
mod nano;
mod prompt;
mod rcfile;
mod search;
mod text;
mod utils;
mod winio;

#[cfg(feature = "browser")]
mod browser;

use clap::Parser;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "rano", version, about = "rano - a Rust rewrite of GNU nano")]
struct Args {
    /// Show line numbers
    #[arg(short = 'l', long = "linenumbers")]
    line_numbers: bool,

    /// Enable mouse support
    #[arg(short = 'm', long = "mouse")]
    mouse: bool,

    /// Display the position of the cursor
    #[arg(short = 'c', long = "constantshow")]
    constant_show: bool,

    /// Automatically indent new lines
    #[arg(short = 'i', long = "autoindent")]
    autoindent: bool,

    /// Enable softwrap
    #[arg(short = 'S', long = "softwrap")]
    softwrap: bool,

    /// Set tab size (in columns)
    #[arg(short = 'T', long = "tabsize", default_value_t = 8)]
    tabsize: usize,

    /// Convert typed tabs to spaces
    #[arg(short = 'E', long = "tabstospaces")]
    tabs_to_spaces: bool,

    /// Don't wrap text at all
    #[arg(short = 'w', long = "nowrap")]
    no_wrap: bool,

    /// Disable the help lines at the bottom
    #[arg(short = 'x', long = "nohelp")]
    no_help: bool,

    /// Use bold instead of reverse video text
    #[arg(short = 'D', long = "boldtext")]
    bold_text: bool,

    /// View mode (read-only)
    #[arg(short = 'v', long = "view")]
    view: bool,

    /// Don't look at nanorc files
    #[arg(short = 'I', long = "ignorercfiles")]
    ignore_rcfiles: bool,

    /// Don't add newlines to the ends of files
    #[arg(short = 'L', long = "nonewlines")]
    no_newlines: bool,

    /// Make backup files
    #[arg(short = 'B', long = "backup")]
    backup: bool,

    /// Start at line,column
    #[arg(short = '+', long = "startpos", value_name = "LINE,COL")]
    start_pos: Option<String>,

    /// Enable syntax highlighting
    #[arg(short = 'Y', long = "syntax", value_name = "NAME")]
    syntax: Option<String>,

    /// Files to edit
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let mut editor = match editor::Editor::new(&args) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("rano: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = editor.run() {
        // Make sure terminal is cleaned up before printing error
        let _ = editor.cleanup();
        eprintln!("rano: {}", e);
        process::exit(1);
    }
}
