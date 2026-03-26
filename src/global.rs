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


/// Global keybindings and function table. Port of nano's global.c.

use crate::definitions::*;

/// Build the default function table (what appears in help and bottombars).
pub fn build_func_table() -> Vec<FuncEntry> {
    let mut funcs = Vec::new();

    // Main menu functions
    let main_funcs = [
        (EditorFunction::DoHelp, "Help", "Invoke the help viewer", Menu::MOST),
        (EditorFunction::DoExit, "Exit", "Close the current buffer / Exit from rano", Menu::MAIN),
        (EditorFunction::DoWriteOut, "Write Out", "Write the current buffer to disk", Menu::MAIN),
        (EditorFunction::DoSearch, "Where Is", "Search for a string or a regular expression", Menu::MAIN),
        (EditorFunction::CutText, "Cut", "Cut current line (or marked region) and store it", Menu::MAIN),
        (EditorFunction::PasteText, "Paste", "Paste the stored text into the current buffer", Menu::MAIN),
        (EditorFunction::DoReplace, "Replace", "Search for and replace a string", Menu::MAIN),
        (EditorFunction::DoGotoLineColumn, "Go To Line", "Go to a specific line and column number", Menu::MAIN),
        (EditorFunction::DoInsertFile, "Read File", "Insert another file into current buffer", Menu::MAIN),
        (EditorFunction::DoSearchForward, "Where Next", "Search next occurrence forward", Menu::MAIN),
        (EditorFunction::DoSearchBackward, "Where Previous", "Search next occurrence backward", Menu::MAIN),
        (EditorFunction::DoSaveFile, "Save", "Save file without prompting", Menu::MAIN),
        (EditorFunction::DoUndo, "Undo", "Undo the last operation", Menu::MAIN),
        (EditorFunction::DoRedo, "Redo", "Redo the last undone operation", Menu::MAIN),
        (EditorFunction::DoMark, "Set Mark", "Mark text starting from the cursor position", Menu::MAIN),
        (EditorFunction::CopyText, "Copy", "Copy current line (or marked region)", Menu::MAIN),
        (EditorFunction::DoIndent, "Indent", "Indent the current line (or marked lines)", Menu::MAIN),
        (EditorFunction::DoUnindent, "Unindent", "Unindent the current line (or marked lines)", Menu::MAIN),
        (EditorFunction::DoComment, "Comment", "Comment/uncomment the current line (or marked lines)", Menu::MAIN),
        (EditorFunction::DoVerbatimInput, "Verbatim", "Insert the next keystroke verbatim", Menu::MAIN),
        (EditorFunction::DoTab, "Tab", "Insert a tab at the cursor position", Menu::MAIN),
        (EditorFunction::DoEnter, "Enter", "Insert a newline at the cursor position", Menu::MAIN),
        (EditorFunction::DoDelete, "Delete", "Delete the character under the cursor", Menu::MAIN),
        (EditorFunction::DoBackspace, "Backspace", "Delete the character before the cursor", Menu::MAIN),
        (EditorFunction::CutTillEof, "Cut Till End", "Cut from cursor to end of file", Menu::MAIN),
        (EditorFunction::ZapText, "Zap", "Throw away the current line (or marked region)", Menu::MAIN),
        (EditorFunction::DoSpell, "Spell Check", "Invoke the spell checker", Menu::MAIN),
        (EditorFunction::DoLinter, "Linter", "Invoke the linter, if available", Menu::MAIN),
        (EditorFunction::DoFormatter, "Formatter", "Invoke the formatter, if available", Menu::MAIN),
    ];

    for (func, tag, phrase, menus) in main_funcs.iter() {
        funcs.push(FuncEntry {
            func: *func,
            tag,
            #[cfg(feature = "help")]
            phrase,
            #[cfg(feature = "help")]
            blank_after: false,
            menus: *menus,
        });
    }

    // Movement functions
    let move_funcs = [
        (EditorFunction::DoUp, "Up", "Move up one line", Menu::MAIN),
        (EditorFunction::DoDown, "Down", "Move down one line", Menu::MAIN),
        (EditorFunction::DoLeft, "Left", "Move back one character", Menu::MAIN),
        (EditorFunction::DoRight, "Right", "Move forward one character", Menu::MAIN),
        (EditorFunction::DoHome, "Home", "Move to the beginning of the line", Menu::MAIN),
        (EditorFunction::DoEnd, "End", "Move to the end of the line", Menu::MAIN),
        (EditorFunction::DoPageUp, "Page Up", "Move up one screenful", Menu::MAIN),
        (EditorFunction::DoPageDown, "Page Down", "Move down one screenful", Menu::MAIN),
        (EditorFunction::ToFirstLine, "First Line", "Go to the first line of the file", Menu::MAIN),
        (EditorFunction::ToLastLine, "Last Line", "Go to the last line of the file", Menu::MAIN),
        (EditorFunction::ToPrevWord, "Prev Word", "Move to the previous word", Menu::MAIN),
        (EditorFunction::ToNextWord, "Next Word", "Move to the next word", Menu::MAIN),
        (EditorFunction::ToPrevBlock, "Prev Block", "Move to the previous block of text", Menu::MAIN),
        (EditorFunction::ToNextBlock, "Next Block", "Move to the next block of text", Menu::MAIN),
        (EditorFunction::DoScrollUp, "Scroll Up", "Scroll up one line without moving cursor", Menu::MAIN),
        (EditorFunction::DoScrollDown, "Scroll Down", "Scroll down one line without moving cursor", Menu::MAIN),
        (EditorFunction::DoCycle, "Center/Top/Bottom", "Center/Top/Bottom the line with the cursor", Menu::MAIN),
    ];

    for (func, tag, phrase, menus) in move_funcs.iter() {
        funcs.push(FuncEntry {
            func: *func,
            tag,
            #[cfg(feature = "help")]
            phrase,
            #[cfg(feature = "help")]
            blank_after: false,
            menus: *menus,
        });
    }

    funcs
}

/// Build the default keybinding table.
pub fn build_keybindings() -> Vec<KeyBinding> {
    let mut keys = Vec::new();

    // Helper to add a binding
    macro_rules! bind {
        ($keystr:expr, $keycode:expr, $menu:expr, $func:expr) => {
            keys.push(KeyBinding {
                keystr: $keystr.to_string(),
                keycode: $keycode,
                menus: $menu,
                func: $func,
                toggle: None,
                ordinal: None,
                expansion: None,
            });
        };
    }

    // Main menu keybindings (nano defaults)
    bind!("^G", KeyCode::Ctrl('g'), Menu::MAIN, EditorFunction::DoHelp);
    bind!("^X", KeyCode::Ctrl('x'), Menu::MAIN, EditorFunction::DoExit);
    bind!("^O", KeyCode::Ctrl('o'), Menu::MAIN, EditorFunction::DoWriteOut);
    bind!("^S", KeyCode::Ctrl('s'), Menu::MAIN, EditorFunction::DoSaveFile);
    bind!("^R", KeyCode::Ctrl('r'), Menu::MAIN, EditorFunction::DoInsertFile);
    bind!("^W", KeyCode::Ctrl('w'), Menu::MAIN, EditorFunction::DoSearch);
    bind!("^\\", KeyCode::Ctrl('\\'), Menu::MAIN, EditorFunction::DoReplace);
    bind!("^K", KeyCode::Ctrl('k'), Menu::MAIN, EditorFunction::CutText);
    bind!("^U", KeyCode::Ctrl('u'), Menu::MAIN, EditorFunction::PasteText);
    bind!("^T", KeyCode::Ctrl('t'), Menu::MAIN, EditorFunction::DoSpell);
    bind!("^J", KeyCode::Ctrl('j'), Menu::MAIN, EditorFunction::DoJustify);
    bind!("^C", KeyCode::Ctrl('c'), Menu::MAIN, EditorFunction::ReportCursorPosition);
    bind!("^_", KeyCode::Ctrl('_'), Menu::MAIN, EditorFunction::DoGotoLineColumn);

    // Movement keys
    bind!("Up", KeyCode::Up, Menu::MAIN, EditorFunction::DoUp);
    bind!("Down", KeyCode::Down, Menu::MAIN, EditorFunction::DoDown);
    bind!("Left", KeyCode::Left, Menu::MAIN, EditorFunction::DoLeft);
    bind!("Right", KeyCode::Right, Menu::MAIN, EditorFunction::DoRight);
    bind!("Home", KeyCode::Home, Menu::MAIN, EditorFunction::DoHome);
    bind!("End", KeyCode::End, Menu::MAIN, EditorFunction::DoEnd);
    bind!("PgUp", KeyCode::PageUp, Menu::MAIN, EditorFunction::DoPageUp);
    bind!("PgDn", KeyCode::PageDown, Menu::MAIN, EditorFunction::DoPageDown);
    bind!("^Home", KeyCode::Special(SpecialKey::ControlHome), Menu::MAIN, EditorFunction::ToFirstLine);
    bind!("^End", KeyCode::Special(SpecialKey::ControlEnd), Menu::MAIN, EditorFunction::ToLastLine);

    // Alt movement
    bind!("M-\\", KeyCode::Alt('\\'), Menu::MAIN, EditorFunction::ToFirstLine);
    bind!("M-/", KeyCode::Alt('/'), Menu::MAIN, EditorFunction::ToLastLine);
    bind!("^Left", KeyCode::Special(SpecialKey::ControlLeft), Menu::MAIN, EditorFunction::ToPrevWord);
    bind!("^Right", KeyCode::Special(SpecialKey::ControlRight), Menu::MAIN, EditorFunction::ToNextWord);
    bind!("M-Up", KeyCode::Special(SpecialKey::AltUp), Menu::MAIN, EditorFunction::ToPrevBlock);
    bind!("M-Down", KeyCode::Special(SpecialKey::AltDown), Menu::MAIN, EditorFunction::ToNextBlock);

    // Scroll
    bind!("M-Up", KeyCode::Alt('p'), Menu::MAIN, EditorFunction::DoScrollUp);
    bind!("M-Down", KeyCode::Alt('n'), Menu::MAIN, EditorFunction::DoScrollDown);

    // Editing
    bind!("^H", KeyCode::Backspace, Menu::MAIN, EditorFunction::DoBackspace);
    bind!("Del", KeyCode::Delete, Menu::MAIN, EditorFunction::DoDelete);
    bind!("Tab", KeyCode::Tab, Menu::MAIN, EditorFunction::DoTab);
    bind!("Enter", KeyCode::Enter, Menu::MAIN, EditorFunction::DoEnter);
    bind!("^V", KeyCode::Ctrl('v'), Menu::MAIN, EditorFunction::DoVerbatimInput);

    // Undo/Redo
    bind!("M-U", KeyCode::Alt('u'), Menu::MAIN, EditorFunction::DoUndo);
    bind!("M-E", KeyCode::Alt('e'), Menu::MAIN, EditorFunction::DoRedo);

    // Mark, Copy
    bind!("M-A", KeyCode::Alt('a'), Menu::MAIN, EditorFunction::DoMark);
    bind!("M-6", KeyCode::Alt('6'), Menu::MAIN, EditorFunction::CopyText);

    // Indent/Comment
    bind!("M-}", KeyCode::Alt('}'), Menu::MAIN, EditorFunction::DoIndent);
    bind!("M-{", KeyCode::Alt('{'), Menu::MAIN, EditorFunction::DoUnindent);
    bind!("M-3", KeyCode::Alt('3'), Menu::MAIN, EditorFunction::DoComment);

    // Search prompt bindings
    bind!("^C", KeyCode::Ctrl('c'), Menu::WHEREIS, EditorFunction::DoCancel);
    bind!("^C", KeyCode::Ctrl('c'), Menu::REPLACE, EditorFunction::DoCancel);
    bind!("^C", KeyCode::Ctrl('c'), Menu::REPLACEWITH, EditorFunction::DoCancel);
    bind!("^C", KeyCode::Ctrl('c'), Menu::GOTOLINE, EditorFunction::DoCancel);

    // Search toggles
    bind!("M-C", KeyCode::Alt('c'), Menu::WHEREIS, EditorFunction::CaseSensVoid);
    bind!("M-R", KeyCode::Alt('r'), Menu::WHEREIS, EditorFunction::RegexpVoid);
    bind!("M-B", KeyCode::Alt('b'), Menu::WHEREIS, EditorFunction::BackwardsVoid);
    bind!("^R", KeyCode::Ctrl('r'), Menu::WHEREIS, EditorFunction::FlipReplace);

    // Search/find
    bind!("M-W", KeyCode::Alt('w'), Menu::MAIN, EditorFunction::DoFindNext);
    bind!("M-Q", KeyCode::Alt('q'), Menu::MAIN, EditorFunction::DoFindPrevious);
    bind!("M-]", KeyCode::Alt(']'), Menu::MAIN, EditorFunction::DoFindBracket);

    // Buffer switching
    bind!("M-<", KeyCode::Alt('<'), Menu::MAIN, EditorFunction::SwitchToPrevBuffer);
    bind!("M->", KeyCode::Alt('>'), Menu::MAIN, EditorFunction::SwitchToNextBuffer);

    // Suspend
    bind!("^Z", KeyCode::Ctrl('z'), Menu::MAIN, EditorFunction::DoSuspend);

    // Word completion
    bind!("^]", KeyCode::Ctrl(']'), Menu::MAIN, EditorFunction::CompleteAWord);

    // Anchors
    bind!("M-Ins", KeyCode::Special(SpecialKey::AltInsert), Menu::MAIN, EditorFunction::PutOrLiftAnchor);

    // Misc prompt
    bind!("^C", KeyCode::Ctrl('c'), Menu::YESNO, EditorFunction::DoCancel);

    keys
}

/// Look up the first shortcut for a given function in a menu.
pub fn first_shortcut_for(
    keybindings: &[KeyBinding],
    menu: Menu,
    func: EditorFunction,
) -> Option<&KeyBinding> {
    keybindings
        .iter()
        .find(|k| k.menus.intersects(menu) && k.func == func)
}

/// Return the number of shown shortcut entries for a given menu.
pub fn shown_entries_for(func_table: &[FuncEntry], menu: Menu) -> usize {
    func_table.iter().filter(|f| f.menus.intersects(menu)).count()
}

/// Look up the function bound to a given keycode in the current menu.
pub fn func_from_key(keybindings: &[KeyBinding], keycode: KeyCode, menu: Menu) -> Option<EditorFunction> {
    keybindings
        .iter()
        .find(|k| k.keycode == keycode && k.menus.intersects(menu))
        .map(|k| k.func)
}

/// Return the human-readable name of an editor flag.
pub fn epithet_of_flag(flag: EditorFlags) -> &'static str {
    if flag == EditorFlags::CASE_SENSITIVE { "Case Sensitive" }
    else if flag == EditorFlags::CONSTANT_SHOW { "Constant Show" }
    else if flag == EditorFlags::NO_HELP { "No Help" }
    else if flag == EditorFlags::NO_WRAP { "No Wrap" }
    else if flag == EditorFlags::AUTOINDENT { "Auto Indent" }
    else if flag == EditorFlags::VIEW_MODE { "View Mode" }
    else if flag == EditorFlags::USE_MOUSE { "Mouse" }
    else if flag == EditorFlags::USE_REGEXP { "Regexp" }
    else if flag == EditorFlags::SOFTWRAP { "Softwrap" }
    else if flag == EditorFlags::LINE_NUMBERS { "Line Numbers" }
    else if flag == EditorFlags::TABS_TO_SPACES { "Tabs to Spaces" }
    else if flag == EditorFlags::MAKE_BACKUP { "Backup" }
    else { "Unknown" }
}
