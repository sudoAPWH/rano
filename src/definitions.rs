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


use bitflags::bitflags;

// ── Constants ──────────────────────────────────────────────────────────

pub const PATH_MAX: usize = 4096;
pub const WIDTH_OF_TAB: usize = 8;
pub const COLUMNS_FROM_EOL: usize = 8;
pub const CUSHION: usize = 3;
pub const GENERAL_COMMENT_CHARACTER: &str = "#";
pub const MAX_SEARCH_HISTORY: usize = 100;
pub const MAXCHARLEN: usize = 4; // UTF-8

pub const ESC_CODE: u8 = 0x1B;
pub const DEL_CODE: u8 = 0x7F;

// ── Direction ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Backward,
    Forward,
}

// ── File format ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatType {
    Unspecified,
    Unix,
    Dos,
    Mac,
}

impl Default for FormatType {
    fn default() -> Self {
        FormatType::Unspecified
    }
}

// ── Message importance ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageType {
    Vacuum,
    Hush,
    Remark,
    Info,
    Notice,
    Ahem,
    Mild,
    Alert,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Vacuum
    }
}

// ── Writing mode ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritingMode {
    Overwrite,
    Append,
    Prepend,
    Emergency,
}

// ── Screen update type ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateType {
    Centering,
    Flowing,
    Stationary,
}

// ── Undo types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoType {
    Add,
    Enter,
    Back,
    Del,
    Join,
    Replace,
    SplitBegin,
    SplitEnd,
    Indent,
    Unindent,
    Comment,
    Uncomment,
    Preflight,
    Zap,
    Cut,
    CutToEof,
    Copy,
    Paste,
    Insert,
    CoupleBegin,
    CoupleEnd,
    Other,
}

// ── Search mode ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    JustFind,
    Replacing,
    InRegion,
}

// ── Ask user result ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskResult {
    Yes,
    All,
    No,
    Cancel,
}

// ── Interface elements for coloring ────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterfaceElement {
    TitleBar,
    LineNumber,
    GuideStripe,
    ScrollBar,
    SelectedText,
    Spotlighted,
    MiniInfobar,
    PromptBar,
    StatusBar,
    ErrorMessage,
    KeyCombo,
    FunctionTag,
}

pub const NUMBER_OF_ELEMENTS: usize = 12;

impl InterfaceElement {
    pub fn index(self) -> usize {
        self as usize
    }
}

// ── Multiline regex flags ──────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MultilineFlags: u8 {
        const NOTHING    = 1 << 1;
        const STARTSHERE = 1 << 2;
        const WHOLELINE  = 1 << 3;
        const ENDSHERE   = 1 << 4;
        const JUSTONTHIS = 1 << 5;
    }
}

// ── Editor flags ───────────────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EditorFlags: u64 {
        const CASE_SENSITIVE      = 1 << 0;
        const CONSTANT_SHOW       = 1 << 1;
        const NO_HELP             = 1 << 2;
        const NO_WRAP             = 1 << 3;
        const AUTOINDENT          = 1 << 4;
        const VIEW_MODE           = 1 << 5;
        const USE_MOUSE           = 1 << 6;
        const USE_REGEXP          = 1 << 7;
        const SAVE_ON_EXIT        = 1 << 8;
        const CUT_FROM_CURSOR     = 1 << 9;
        const BACKWARDS_SEARCH    = 1 << 10;
        const MULTIBUFFER         = 1 << 11;
        const REBIND_DELETE       = 1 << 12;
        const RAW_SEQUENCES       = 1 << 13;
        const NO_CONVERT          = 1 << 14;
        const MAKE_BACKUP         = 1 << 15;
        const INSECURE_BACKUP     = 1 << 16;
        const NO_SYNTAX           = 1 << 17;
        const PRESERVE            = 1 << 18;
        const HISTORYLOG          = 1 << 19;
        const RESTRICTED          = 1 << 20;
        const SMART_HOME          = 1 << 21;
        const WHITESPACE_DISPLAY  = 1 << 22;
        const TABS_TO_SPACES      = 1 << 23;
        const QUICK_BLANK         = 1 << 24;
        const WORD_BOUNDS         = 1 << 25;
        const NO_NEWLINES         = 1 << 26;
        const BOLD_TEXT           = 1 << 27;
        const SOFTWRAP            = 1 << 28;
        const POSITIONLOG         = 1 << 29;
        const LOCKING             = 1 << 30;
        const NOREAD_MODE         = 1 << 31;
        const MAKE_IT_UNIX        = 1 << 32;
        const TRIM_BLANKS         = 1 << 33;
        const SHOW_CURSOR         = 1 << 34;
        const LINE_NUMBERS        = 1 << 35;
        const AT_BLANKS           = 1 << 36;
        const AFTER_ENDS          = 1 << 37;
        const LET_THEM_ZAP        = 1 << 38;
        const BREAK_LONG_LINES    = 1 << 39;
        const JUMPY_SCROLLING     = 1 << 40;
        const EMPTY_LINE          = 1 << 41;
        const INDICATOR           = 1 << 42;
        const BOOKSTYLE           = 1 << 43;
        const COLON_PARSING       = 1 << 44;
        const STATEFLAGS          = 1 << 45;
        const USE_MAGIC           = 1 << 46;
        const MINIBAR             = 1 << 47;
        const ZERO                = 1 << 48;
        const MODERN_BINDINGS     = 1 << 49;
        const SOLO_SIDESCROLL     = 1 << 50;
    }
}

// ── Menu identifiers ───────────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Menu: u16 {
        const MAIN         = 1 << 0;
        const WHEREIS      = 1 << 1;
        const REPLACE      = 1 << 2;
        const REPLACEWITH  = 1 << 3;
        const GOTOLINE     = 1 << 4;
        const WRITEFILE    = 1 << 5;
        const INSERTFILE   = 1 << 6;
        const EXECUTE      = 1 << 7;
        const HELP         = 1 << 8;
        const SPELL        = 1 << 9;
        const BROWSER      = 1 << 10;
        const WHEREISFILE  = 1 << 11;
        const GOTODIR      = 1 << 12;
        const YESNO        = 1 << 13;
        const LINTER       = 1 << 14;
        const FINDINHELP   = 1 << 15;
    }
}

impl Menu {
    pub const MOST: Menu = Menu::from_bits_truncate(
        Menu::MAIN.bits()
            | Menu::WHEREIS.bits()
            | Menu::REPLACE.bits()
            | Menu::REPLACEWITH.bits()
            | Menu::GOTOLINE.bits()
            | Menu::WRITEFILE.bits()
            | Menu::INSERTFILE.bits()
            | Menu::EXECUTE.bits()
            | Menu::WHEREISFILE.bits()
            | Menu::GOTODIR.bits()
            | Menu::FINDINHELP.bits()
            | Menu::SPELL.bits()
            | Menu::LINTER.bits(),
    );
}

// ── Undo extra flags ──────────────────────────────────────────────────

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UndoFlags: u8 {
        const WAS_BACKSPACE_AT_EOF = 1 << 1;
        const WAS_WHOLE_LINE       = 1 << 2;
        const INCLUDED_LAST_LINE   = 1 << 3;
        const MARK_WAS_SET         = 1 << 4;
        const CURSOR_WAS_AT_HEAD   = 1 << 5;
        const HAD_ANCHOR_AT_START  = 1 << 6;
    }
}

// ── Custom key codes ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpecialKey {
    ControlLeft,
    ControlRight,
    ControlUp,
    ControlDown,
    ControlHome,
    ControlEnd,
    ControlDelete,
    ShiftControlLeft,
    ShiftControlRight,
    ShiftControlUp,
    ShiftControlDown,
    ShiftControlHome,
    ShiftControlEnd,
    ControlShiftDelete,
    AltLeft,
    AltRight,
    AltUp,
    AltDown,
    AltHome,
    AltEnd,
    AltPageUp,
    AltPageDown,
    AltInsert,
    AltDelete,
    ShiftAltLeft,
    ShiftAltRight,
    ShiftAltUp,
    ShiftAltDown,
    ShiftUp,
    ShiftDown,
    ShiftHome,
    ShiftEnd,
    ShiftPageUp,
    ShiftPageDown,
    ShiftDelete,
    ShiftTab,
    FocusIn,
    FocusOut,
    StartOfPaste,
    EndOfPaste,
    MorePlants,
    MissingBrace,
    PlantedACommand,
    NoSuchFunction,
    KeyCenter,
    WindowResized,
    ForeignSequence,
    KeyFresh,
    MouseScrollUp,
    MouseScrollDown,
    MouseClick(u16, u16),
    MouseDrag(u16, u16),
    PastedText(String),
}

// ── Core data structures ───────────────────────────────────────────────

/// A single line of text in the buffer (replaces linestruct).
/// Instead of a doubly-linked list, we use a Vec<Line> in the Buffer.
#[derive(Debug, Clone)]
pub struct Line {
    pub data: String,
    pub lineno: usize,
    #[cfg(feature = "color")]
    pub multidata: Vec<i16>,
    pub has_anchor: bool,
}

impl Line {
    pub fn new(data: String, lineno: usize) -> Self {
        Line {
            data,
            lineno,
            #[cfg(feature = "color")]
            multidata: Vec::new(),
            has_anchor: false,
        }
    }
}

impl Default for Line {
    fn default() -> Self {
        Line::new(String::new(), 0)
    }
}

/// A group of lines affected by an indent/unindent operation.
#[derive(Debug, Clone)]
pub struct UndoGroup {
    pub top_line: usize,
    pub bottom_line: usize,
    pub indentations: Vec<String>,
}

/// An undo history item (replaces undostruct).
#[derive(Debug, Clone)]
pub struct UndoItem {
    pub undo_type: UndoType,
    pub xflags: UndoFlags,
    pub head_lineno: usize,
    pub head_x: usize,
    pub strdata: Option<String>,
    pub wassize: usize,
    pub newsize: usize,
    pub grouping: Vec<UndoGroup>,
    pub cutbuffer: Vec<Line>,
    pub tail_lineno: usize,
    pub tail_x: usize,
}

/// Syntax highlighting color definition (replaces colortype).
#[cfg(feature = "color")]
#[derive(Debug, Clone)]
pub struct ColorRule {
    pub id: i16,
    pub fg: i16,
    pub bg: i16,
    pub attributes: crossterm::style::Attributes,
    pub start: regex::Regex,
    pub end: Option<regex::Regex>,
}

/// A regex list entry (replaces regexlisttype).
#[cfg(feature = "color")]
#[derive(Debug, Clone)]
pub struct RegexEntry {
    pub regex: regex::Regex,
}

/// Syntax definition (replaces syntaxtype).
#[cfg(feature = "color")]
#[derive(Debug, Clone)]
pub struct SyntaxDef {
    pub name: String,
    pub filename: Option<String>,
    pub lineno: usize,
    pub extensions: Vec<RegexEntry>,
    pub headers: Vec<RegexEntry>,
    pub magics: Vec<RegexEntry>,
    pub linter: Option<String>,
    pub formatter: Option<String>,
    pub tabstring: Option<String>,
    #[cfg(feature = "comment")]
    pub comment: Option<String>,
    pub colors: Vec<ColorRule>,
    pub multiscore: i16,
}

/// A lint error.
#[cfg(feature = "linter")]
#[derive(Debug, Clone)]
pub struct LintMessage {
    pub lineno: usize,
    pub colno: usize,
    pub msg: String,
    pub filename: String,
}

/// A position in a file for the position history.
#[cfg(feature = "histories")]
#[derive(Debug, Clone)]
pub struct PositionEntry {
    pub filename: String,
    pub linenumber: usize,
    pub columnnumber: usize,
    pub anchors: String,
}

/// An open file buffer (replaces openfilestruct).
#[derive(Debug)]
pub struct OpenBuffer {
    pub filename: String,
    pub lines: Vec<Line>,
    pub edittop: usize,         // index into lines
    pub current: usize,         // index into lines
    pub totsize: usize,
    pub firstcolumn: usize,
    pub current_x: usize,
    pub placewewant: usize,
    pub brink: usize,
    pub cursor_row: isize,
    pub modified: bool,
    pub format: FormatType,
    pub mark: Option<usize>,        // line index where mark is set
    pub mark_x: usize,
    pub softmark: bool,
    pub drag_end: Option<usize>,     // line index of drag selection endpoint
    pub drag_end_x: usize,
    pub saved_current: usize,        // cursor position saved before mouse drag
    pub saved_current_x: usize,
    pub saved_placewewant: usize,
    pub undo_stack: Vec<UndoItem>,
    pub undo_index: usize,          // points past the last valid undo
    pub last_action: Option<UndoType>,
    pub last_saved_index: usize,
    #[cfg(feature = "color")]
    pub syntax: Option<usize>,      // index into syntax list
    pub lock_filename: Option<String>,
    pub errormessage: Option<String>,
    #[cfg(feature = "wrapping")]
    pub spillage_line: Option<usize>,
}

impl OpenBuffer {
    pub fn new() -> Self {
        OpenBuffer {
            filename: String::new(),
            lines: vec![Line::new(String::new(), 1)],
            edittop: 0,
            current: 0,
            totsize: 0,
            firstcolumn: 0,
            current_x: 0,
            placewewant: 0,
            brink: 0,
            cursor_row: 0,
            modified: false,
            format: FormatType::default(),
            mark: None,
            mark_x: 0,
            softmark: false,
            drag_end: None,
            drag_end_x: 0,
            saved_current: 0,
            saved_current_x: 0,
            saved_placewewant: 0,
            undo_stack: Vec::new(),
            undo_index: 0,
            last_action: None,
            last_saved_index: 0,
            #[cfg(feature = "color")]
            syntax: None,
            lock_filename: None,
            errormessage: None,
            #[cfg(feature = "wrapping")]
            spillage_line: None,
        }
    }

    pub fn current_line(&self) -> &Line {
        &self.lines[self.current]
    }

    pub fn current_line_mut(&mut self) -> &mut Line {
        &mut self.lines[self.current]
    }

    /// Renumber all lines starting from the given index.
    pub fn renumber_from(&mut self, start: usize) {
        for i in start..self.lines.len() {
            self.lines[i].lineno = i + 1;
        }
    }

    /// Total number of lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

/// A keybinding (replaces keystruct).
#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub keystr: String,
    pub keycode: KeyCode,
    pub menus: Menu,
    pub func: EditorFunction,
    pub toggle: Option<u32>,
    pub ordinal: Option<u32>,
    pub expansion: Option<String>,
}

/// An editor function reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorFunction {
    // Main editing
    DoExit,
    DoInsertFile,
    DoSearch,
    DoReplace,
    DoWriteOut,
    DoSaveFile,
    CutText,
    PasteText,
    CopyText,
    DoUndo,
    DoRedo,
    DoEnter,
    DoTab,
    DoDelete,
    DoBackspace,
    DoVerbatimInput,
    DoMark,
    DoComment,
    DoIndent,
    DoUnindent,
    DoJustify,
    DoFullJustify,
    DoSpell,
    DoLinter,
    DoFormatter,
    CutTillEof,
    ZapText,
    CopyMarkedRegion,
    // Movement
    DoUp,
    DoDown,
    DoLeft,
    DoRight,
    DoHome,
    DoEnd,
    DoPageUp,
    DoPageDown,
    ToFirstLine,
    ToLastLine,
    ToPrevWord,
    ToNextWord,
    ToPrevBlock,
    ToNextBlock,
    DoScrollUp,
    DoScrollDown,
    DoScrollLeft,
    DoScrollRight,
    ToTopRow,
    ToBottomRow,
    DoCycle,
    DoCenter,
    // Search
    DoSearchForward,
    DoSearchBackward,
    DoFindPrevious,
    DoFindNext,
    DoGotoLineColumn,
    DoFindBracket,
    // Anchors
    PutOrLiftAnchor,
    ToPrevAnchor,
    ToNextAnchor,
    // Prompt
    CaseSensVoid,
    RegexpVoid,
    BackwardsVoid,
    FlipReplace,
    FlipGoto,
    FlipExecute,
    FlipPipe,
    FlipConvert,
    FlipNewbuffer,
    GetOlderItem,
    GetNewerItem,
    DiscardBuffer,
    DoCancel,
    // Buffer management
    SwitchToPrevBuffer,
    SwitchToNextBuffer,
    CloseBuffer,
    // Browser
    ToFiles,
    GotoDir,
    ToFirstFile,
    ToLastFile,
    // Help
    DoHelp,
    // Toggle
    DoToggle,
    DoNothing,
    // Format
    DosFormat,
    MacFormat,
    AppendIt,
    PrependIt,
    BackItUp,
    // Misc
    DoSuspend,
    ReportCursorPosition,
    CopyOrPosition,
    DoCredits,
    CountLinesWordsAndCharacters,
    CompleteAWord,
    RecordMacro,
    RunMacro,
    ChopPreviousWord,
    ChopNextWord,
    // For prompt add/remove pipe
    AddOrRemovePipeSymbol,
    AskForLineAndColumn,
    SuckUpInputAndPasteIt,
}

/// A function table entry (replaces funcstruct).
#[derive(Debug, Clone)]
pub struct FuncEntry {
    pub func: EditorFunction,
    pub tag: &'static str,
    #[cfg(feature = "help")]
    pub phrase: &'static str,
    #[cfg(feature = "help")]
    pub blank_after: bool,
    pub menus: Menu,
}

/// A word completion entry.
#[cfg(feature = "wordcompletion")]
#[derive(Debug, Clone)]
pub struct CompletionEntry {
    pub word: String,
}

/// Rcfile option definition.
#[derive(Debug, Clone)]
pub struct RcOption {
    pub name: &'static str,
    pub flag: Option<EditorFlags>,
}

// ── Key representation ─────────────────────────────────────────────────

/// Our unified key code representation, mapped from crossterm events.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Char(char),
    Ctrl(char),
    Alt(char),
    AltCtrl(char),
    F(u8),
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    Backspace,
    Enter,
    Tab,
    Escape,
    Special(SpecialKey),
    Mouse,
    Null,
    Unknown(u32),
}
