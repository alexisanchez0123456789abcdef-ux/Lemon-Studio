// Copyright (C) 2026 ItsAlphaNeon
// SPDX-License-Identifier: GPL-3.0-or-later
// This file is part of Lemon Studio: https://github.com/ItsAlphaNeon/Lemon-Studio

//! The interface's language: every label the UI draws goes through [`t`], which hands back the text in the language
//! General ▸ Language chose (`crate::prefs::LANGUAGE`), or the text itself when the table has no entry for it.
//!
//! A label's English text is its key: a control's label, a menu entry, a hint, a group title. A label is translated
//! where it is drawn, never where it is used as a key (an action, a `settings.txt` value or a hit region keeps the
//! English), so a language change renames what is shown and nothing else. Text the user typed (channel, pattern, sample
//! and project names) is drawn as it is: it is not a key.
//!
//! The tables live in `assets/lang/<tag>.txt`, one `English<TAB>Translation` per line; `\t`, `\n` and `\\` are escapes,
//! a line starting with `#` is a comment. English is the source text itself, so it has no table. A line whose key is
//! already in the table replaces it.
// spec: ui/settings.md "General" (Language)

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

/// A language the interface can be drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    Spanish,
}

impl Language {
    /// Every language, in General ▸ Language's order (the combo's choice is one of these indexes).
    pub const ALL: [Language; 2] = [Language::English, Language::Spanish];

    /// The language's own name, as General ▸ Language lists it (endonyms are never translated).
    pub const fn name(self) -> &'static str {
        match self {
            Language::English => "English (US) (en)",
            Language::Spanish => "Español (es)",
        }
    }

    /// The language's table (`assets/lang/<tag>.txt`); English has none, being the source text.
    fn source(self) -> Option<&'static str> {
        match self {
            Language::English => None,
            Language::Spanish => Some(include_str!("../assets/lang/es.txt")),
        }
    }

    /// The language at `index` in [`Language::ALL`]; past the end, English.
    pub fn from_index(index: usize) -> Language {
        Self::ALL.get(index).copied().unwrap_or(Language::English)
    }

    /// Its index in [`Language::ALL`].
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|l| *l == self).unwrap_or(0)
    }
}

/// The language being drawn in (English until General ▸ Language says otherwise).
static CURRENT: AtomicU8 = AtomicU8::new(0);

/// `t`'s table, built from the language's file the first time a label is translated in it.
static TABLES: OnceLock<HashMap<&'static str, String>> = OnceLock::new();

/// The language the interface is drawn in.
pub fn language() -> Language {
    Language::from_index(usize::from(CURRENT.load(Ordering::Relaxed)))
}

/// Draws the interface in `language` from the next label on: a repaint shows the whole UI in it.
pub fn set(language: Language) {
    CURRENT.store(language.index() as u8, Ordering::Relaxed);
}

/// `text` in the interface's language: its translation when the table has one, `text` itself otherwise.
pub fn t(text: &str) -> &str {
    let language = language();
    if language == Language::English {
        return text;
    }
    table(language).get(text).map_or(text, String::as_str)
}

/// `text` in the interface's language, owned: for a label a menu or a list keeps.
pub fn owned(text: &str) -> String {
    t(text).to_string()
}

fn table(language: Language) -> &'static HashMap<&'static str, String> {
    TABLES.get_or_init(|| language.source().map(parse).unwrap_or_default())
}

/// Parses a language file: `English<TAB>Translation` per line, `#` for a comment.
fn parse(source: &'static str) -> HashMap<&'static str, String> {
    let mut entries: HashMap<&'static str, String> = HashMap::new();
    let mut offset = 0;
    for raw in source.split_inclusive('\n') {
        let line = raw.trim_end_matches('\n').trim_end_matches('\r');
        let start = offset;
        offset += raw.len();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('\t') else {
            debug_assert!(false, "language line without a tab: {line:?}");
            continue;
        };
        // The key is a slice of the file, so it lives as long as the table does; only the value is unescaped.
        let key = &source[start..start + key.len()];
        debug_assert!(!entries.contains_key(key), "language line translated twice: {key:?}");
        entries.insert(key, unescape(value.trim_end()));
    }
    entries
}

/// `\t`, `\n` and `\\` in a table's value.
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every table parses, with no line missing its tab (a debug assertion catches the malformed ones).
    #[test]
    fn tables_parse() {
        for language in Language::ALL {
            let Some(source) = language.source() else { continue };
            assert!(!source.is_empty(), "{} has no table", language.name());
            for line in source.lines() {
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                assert!(line.contains('\t'), "{}: {line:?} has no tab", language.name());
            }
        }
    }

    /// The escapes of a value.
    #[test]
    fn escapes() {
        assert_eq!(unescape("a\\tb"), "a\tb");
        assert_eq!(unescape("a\\nb"), "a\nb");
        assert_eq!(unescape("a\\\\b"), "a\\b");
        assert_eq!(unescape("a\\qb"), "a\\qb");
        assert_eq!(unescape("ab\\"), "ab\\");
    }

    /// The Spanish table has the entries the interface needs first (the Settings window's).
    #[test]
    fn spanish_table_has_entries() {
        let spanish = table(Language::Spanish);
        assert_eq!(spanish.get("Language").map(String::as_str), Some("Idioma"));
        assert_eq!(spanish.get("Display").map(String::as_str), Some("Pantalla"));
        assert!(spanish.len() > 100, "{} entries", spanish.len());
    }

    /// English returns the text as it is; a language without an entry falls back to English.
    #[test]
    fn english_is_the_source() {
        set(Language::English);
        assert_eq!(t("Language"), "Language");
        assert_eq!(language(), Language::English);
        let spanish = Language::Spanish.index();
        assert_eq!(Language::from_index(spanish), Language::Spanish);
        assert_eq!(Language::from_index(99), Language::English);
    }
}
