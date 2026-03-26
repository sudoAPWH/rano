/// The central Editor struct and its main loop. This ties everything together.

use crate::chars;
use crate::definitions::*;
use crate::global;
use crate::winio;

use crossterm::terminal;
use std::io;

/// The main editor state.
pub struct Editor {
    /// All open file buffers.
    pub buffers: Vec<OpenBuffer>,
    /// Index of the currently active buffer.
    pub current_buf: usize,
    /// Editor-wide flags.
    pub flags: EditorFlags,
    /// Tab size in columns.
    pub tabsize: usize,
    /// Height of the edit window in rows.
    pub editwinrows: usize,
    /// Width of the edit window in columns.
    pub editwincols: usize,
    /// Total terminal rows.
    pub term_rows: usize,
    /// Total terminal columns.
    pub term_cols: usize,
    /// Width of the line-number margin.
    pub margin: usize,
    /// Width of the scrollbar sidebar.
    pub sidebar: usize,
    /// Whether the screen needs a full refresh.
    pub refresh_needed: bool,
    /// The last displayed status message.
    pub lastmessage: MessageType,
    /// The status message text.
    pub statusmsg: String,
    /// Whether we are in the help viewer.
    pub inhelp: bool,
    /// Whether the editor is still running.
    pub running: bool,
    /// Whether focusing is active.
    pub focusing: bool,
    /// The answer in prompt mode.
    pub answer: String,
    /// Last search string.
    pub last_search: String,
    /// The cut buffer.
    pub cutbuffer: Vec<Line>,
    /// Whether to keep the cutbuffer on next cut.
    pub keep_cutbuffer: bool,
    /// Current menu context.
    pub currmenu: Menu,
    /// The function table.
    pub func_table: Vec<FuncEntry>,
    /// The keybinding table.
    pub keybindings: Vec<KeyBinding>,
    /// Home directory.
    pub homedir: Option<String>,
    /// The terminal I/O handler.
    pub tio: winio::TermIO,
    /// Word chars for word detection.
    pub word_chars: Option<String>,
    /// Search flags.
    pub search_flags: EditorFlags,
    #[cfg(feature = "color")]
    pub syntaxes: Vec<crate::definitions::SyntaxDef>,
    #[cfg(feature = "color")]
    pub have_palette: bool,
    #[cfg(feature = "histories")]
    pub search_history: Vec<String>,
    #[cfg(feature = "histories")]
    pub replace_history: Vec<String>,
}

impl Editor {
    /// Create a new editor from command-line arguments.
    pub fn new(args: &crate::Args) -> io::Result<Self> {
        let (cols, rows) = terminal::size()?;
        let term_rows = rows as usize;
        let term_cols = cols as usize;

        // Compute edit window dimensions (leaving room for title bar and status/help bars)
        let help_rows = if args.no_help { 0 } else { 2 };
        let editwinrows = term_rows.saturating_sub(2 + help_rows); // title + status + help
        let editwincols = term_cols;

        let mut flags = EditorFlags::empty();
        if args.line_numbers { flags |= EditorFlags::LINE_NUMBERS; }
        if args.mouse { flags |= EditorFlags::USE_MOUSE; }
        if args.constant_show { flags |= EditorFlags::CONSTANT_SHOW; }
        if args.autoindent { flags |= EditorFlags::AUTOINDENT; }
        if args.softwrap { flags |= EditorFlags::SOFTWRAP; }
        if args.tabs_to_spaces { flags |= EditorFlags::TABS_TO_SPACES; }
        if args.no_wrap { flags |= EditorFlags::NO_WRAP; }
        if args.no_help { flags |= EditorFlags::NO_HELP; }
        if args.bold_text { flags |= EditorFlags::BOLD_TEXT; }
        if args.view { flags |= EditorFlags::VIEW_MODE; }
        if args.no_newlines { flags |= EditorFlags::NO_NEWLINES; }
        if args.backup { flags |= EditorFlags::MAKE_BACKUP; }

        let margin = if flags.contains(EditorFlags::LINE_NUMBERS) { 0 } else { 0 };

        let func_table = global::build_func_table();
        let keybindings = global::build_keybindings();
        let homedir = crate::utils::get_homedir();

        let tio = winio::TermIO::new()?;

        let mut editor = Editor {
            buffers: Vec::new(),
            current_buf: 0,
            flags,
            tabsize: args.tabsize,
            editwinrows,
            editwincols,
            term_rows,
            term_cols,
            margin,
            sidebar: 0,
            refresh_needed: true,
            lastmessage: MessageType::Vacuum,
            statusmsg: String::new(),
            inhelp: false,
            running: true,
            focusing: false,
            answer: String::new(),
            last_search: String::new(),
            cutbuffer: Vec::new(),
            keep_cutbuffer: false,
            currmenu: Menu::MAIN,
            func_table,
            keybindings,
            homedir,
            tio,
            word_chars: None,
            search_flags: EditorFlags::empty(),
            #[cfg(feature = "color")]
            syntaxes: Vec::new(),
            #[cfg(feature = "color")]
            have_palette: false,
            #[cfg(feature = "histories")]
            search_history: Vec::new(),
            #[cfg(feature = "histories")]
            replace_history: Vec::new(),
        };

        // Open files from args, or create an empty buffer
        if args.files.is_empty() {
            editor.make_new_buffer();
        } else {
            for filename in &args.files {
                editor.open_buffer(filename)?;
            }
        }

        // Handle +line,col
        if let Some(ref pos) = args.start_pos {
            let (line, col) = crate::utils::parse_line_column(pos);
            let target_line = line.unwrap_or(1).max(1) as usize - 1;
            let target_col = col.unwrap_or(1).max(1) as usize - 1;
            let buf = &mut editor.buffers[editor.current_buf];
            buf.current = target_line.min(buf.lines.len() - 1);
            buf.current_x = chars::actual_x(
                &buf.lines[buf.current].data,
                target_col,
                editor.tabsize,
            );
        }

        editor.confirm_margin();

        Ok(editor)
    }

    /// Get a reference to the current buffer.
    pub fn current_buffer(&self) -> &OpenBuffer {
        &self.buffers[self.current_buf]
    }

    /// Get a mutable reference to the current buffer.
    pub fn current_buffer_mut(&mut self) -> &mut OpenBuffer {
        &mut self.buffers[self.current_buf]
    }

    /// Create a new empty buffer and switch to it.
    pub fn make_new_buffer(&mut self) {
        let buf = OpenBuffer::new();
        self.buffers.push(buf);
        self.current_buf = self.buffers.len() - 1;
    }

    /// Confirm the line-number margin width.
    pub fn confirm_margin(&mut self) {
        if self.flags.contains(EditorFlags::LINE_NUMBERS) {
            let line_count = self.current_buffer().lines.len();
            self.margin = crate::utils::digits(line_count) + 1;
        } else {
            self.margin = 0;
        }
        self.editwincols = self.term_cols.saturating_sub(self.margin + self.sidebar);
    }

    /// Run the main editor loop.
    pub fn run(&mut self) -> io::Result<()> {
        self.tio.enter_raw_mode()?;
        self.tio.setup_screen()?;

        // Clear any startup messages so the status bar starts blank
        self.statusmsg.clear();
        self.lastmessage = MessageType::Vacuum;
        self.refresh_needed = true;

        while self.running {
            if self.refresh_needed {
                self.full_refresh()?;
                self.refresh_needed = false;
            }

            // Place cursor
            self.place_the_cursor()?;

            // Read input
            let keycode = self.tio.get_kbinput()?;

            // Process the key
            self.process_key(keycode)?;
        }

        self.cleanup()
    }

    /// Clean up terminal state.
    pub fn cleanup(&mut self) -> io::Result<()> {
        self.tio.cleanup()
    }

    /// Process a single keypress.
    fn process_key(&mut self, keycode: KeyCode) -> io::Result<()> {
        // Handle mouse scroll events (3 lines per tick, like nano)
        match keycode {
            KeyCode::Special(SpecialKey::MouseScrollUp) => {
                for _ in 0..3 {
                    self.do_scroll_up();
                }
                self.refresh_needed = true;
                return Ok(());
            }
            KeyCode::Special(SpecialKey::MouseScrollDown) => {
                for _ in 0..3 {
                    self.do_scroll_down();
                }
                self.refresh_needed = true;
                return Ok(());
            }
            _ => {}
        }

        // Check if it's a bound function
        if let Some(func) = global::func_from_key(&self.keybindings, keycode, self.currmenu) {
            self.execute_function(func)?;
            return Ok(());
        }

        // Otherwise, if we're in the main menu, it's a character insertion
        if self.currmenu == Menu::MAIN {
            if let KeyCode::Char(c) = keycode {
                if !self.flags.contains(EditorFlags::VIEW_MODE) {
                    self.do_char(c);
                }
            }
        }

        Ok(())
    }

    /// Execute an editor function.
    fn execute_function(&mut self, func: EditorFunction) -> io::Result<()> {
        match func {
            // Exit
            EditorFunction::DoExit => self.do_exit()?,
            EditorFunction::DoWriteOut => self.do_writeout()?,
            EditorFunction::DoSaveFile => self.do_savefile()?,

            // Movement
            EditorFunction::DoUp => self.do_up(),
            EditorFunction::DoDown => self.do_down(),
            EditorFunction::DoLeft => self.do_left(),
            EditorFunction::DoRight => self.do_right(),
            EditorFunction::DoHome => self.do_home(),
            EditorFunction::DoEnd => self.do_end(),
            EditorFunction::DoPageUp => self.do_page_up(),
            EditorFunction::DoPageDown => self.do_page_down(),
            EditorFunction::ToFirstLine => self.to_first_line(),
            EditorFunction::ToLastLine => self.to_last_line(),
            EditorFunction::ToPrevWord => self.to_prev_word(),
            EditorFunction::ToNextWord => self.to_next_word(),
            EditorFunction::ToPrevBlock => self.to_prev_block(),
            EditorFunction::ToNextBlock => self.to_next_block(),
            EditorFunction::DoScrollUp => self.do_scroll_up(),
            EditorFunction::DoScrollDown => self.do_scroll_down(),

            // Editing
            EditorFunction::DoBackspace => self.do_backspace(),
            EditorFunction::DoDelete => self.do_delete(),
            EditorFunction::DoEnter => self.do_enter(),
            EditorFunction::DoTab => self.do_tab(),
            EditorFunction::CutText => self.cut_text(),
            EditorFunction::PasteText => self.paste_text(),
            EditorFunction::CopyText => self.copy_text(),
            EditorFunction::DoUndo => self.do_undo(),
            EditorFunction::DoRedo => self.do_redo(),
            EditorFunction::DoMark => self.do_mark(),
            EditorFunction::DoIndent => self.do_indent(),
            EditorFunction::DoUnindent => self.do_unindent(),
            EditorFunction::DoComment => self.do_comment(),

            // Search
            EditorFunction::DoSearch | EditorFunction::DoSearchForward => {
                self.do_search(Direction::Forward)?;
            }
            EditorFunction::DoSearchBackward => {
                self.do_search(Direction::Backward)?;
            }
            EditorFunction::DoFindNext => self.do_findnext()?,
            EditorFunction::DoFindPrevious => self.do_findprevious()?,
            EditorFunction::DoReplace => self.do_replace()?,
            EditorFunction::DoGotoLineColumn => self.do_gotolinecolumn()?,

            // File
            EditorFunction::DoInsertFile => self.do_insertfile()?,

            // Help
            EditorFunction::DoHelp => self.do_help()?,

            // Report position
            EditorFunction::ReportCursorPosition => self.report_cursor_position(),

            // Suspend
            EditorFunction::DoSuspend => self.do_suspend()?,

            // Cancel (in prompts)
            EditorFunction::DoCancel => {
                // Return to main menu
                self.currmenu = Menu::MAIN;
                self.refresh_needed = true;
            }

            // Buffer switching
            EditorFunction::SwitchToPrevBuffer => self.switch_to_prev_buffer(),
            EditorFunction::SwitchToNextBuffer => self.switch_to_next_buffer(),

            // Unhandled
            _ => {
                self.statusmsg = format!("Unbound function");
                self.lastmessage = MessageType::Ahem;
            }
        }
        Ok(())
    }

    /// Set the status message.
    pub fn statusline(&mut self, importance: MessageType, msg: &str) {
        if importance >= self.lastmessage {
            self.statusmsg = msg.to_string();
            self.lastmessage = importance;
        }
    }

    /// Mark the current buffer as modified.
    pub fn set_modified(&mut self) {
        self.buffers[self.current_buf].modified = true;
    }

    /// Report cursor position on the status bar.
    pub fn report_cursor_position(&mut self) {
        let buf = &self.buffers[self.current_buf];
        let lineno = buf.current + 1;
        let col = chars::wideness(&buf.lines[buf.current].data, buf.current_x, self.tabsize) + 1;
        let total = buf.lines.len();
        let pct = if total == 0 { 0 } else { lineno * 100 / total };
        self.statusmsg = format!("line {}/{} ({}%), col {}", lineno, total, pct, col);
        self.lastmessage = MessageType::Info;
        self.refresh_needed = true;
    }

    /// Switch to previous buffer.
    pub fn switch_to_prev_buffer(&mut self) {
        if self.buffers.len() <= 1 {
            self.statusline(MessageType::Ahem, "No more open file buffers");
            return;
        }
        if self.current_buf == 0 {
            self.current_buf = self.buffers.len() - 1;
        } else {
            self.current_buf -= 1;
        }
        self.confirm_margin();
        self.refresh_needed = true;
        let name = self.buffers[self.current_buf].filename.clone();
        self.statusline(MessageType::Info, &format!("Switched to {}", if name.is_empty() { "New Buffer" } else { &name }));
    }

    /// Switch to next buffer.
    pub fn switch_to_next_buffer(&mut self) {
        if self.buffers.len() <= 1 {
            self.statusline(MessageType::Ahem, "No more open file buffers");
            return;
        }
        self.current_buf = (self.current_buf + 1) % self.buffers.len();
        self.confirm_margin();
        self.refresh_needed = true;
        let name = self.buffers[self.current_buf].filename.clone();
        self.statusline(MessageType::Info, &format!("Switched to {}", if name.is_empty() { "New Buffer" } else { &name }));
    }

    /// Suspend the editor (send SIGTSTP).
    pub fn do_suspend(&mut self) -> io::Result<()> {
        self.tio.cleanup()?;
        unsafe {
            libc::kill(libc::getpid(), libc::SIGTSTP);
        }
        // When we come back from suspension, re-enter raw mode
        self.tio.enter_raw_mode()?;
        self.tio.setup_screen()?;
        self.refresh_needed = true;
        Ok(())
    }
}
