# Translations

Lemon Studio's interface is drawn in the language chosen in **Settings ▸ General ▸ Language** (English or Spanish
today). The choice is kept as `language=` in the user folder's `settings.txt` and applies at once, with no restart.

## How a label is translated

A label is translated **where it is drawn**, never where it is used as a key. `os_ui::lang::t(text)` returns `text` in
the language chosen, or `text` itself when its table has no line for it:

```rust
ui.c.text(font, x, y, colour, os_ui::lang::t(label));
```

What a control answers to stays English whatever the language:

- its action (`Action::Settings(SAct::Check(label))`), its hit region and its `settings.txt` value;
- the width a row was laid out from, where the layout is not the label's (a fixed-size box);
- what the user typed: channel, pattern, sample and project names are drawn as they are, never looked up.

Where a label is measured before it is drawn (a centred or right-aligned one), translate first and measure the
translated text, so the alignment holds in every language.

Text built at run time is translated in pieces rather than as a whole sentence:

```rust
format!("{} {date}", os_ui::lang::t("This project was started on"))
```

## The tables

The tables live in `crates/os-ui/assets/lang/`, one file per language (`es.txt` today), one entry per line:

```
English text	Spanish text
```

- The key is the exact text the code draws in English: a label, a menu entry, a hint, a group title.
- `\t`, `\n` and `\\` are escapes; a line starting with `#` is a comment.
- A label with no line keeps its English, so a table grows piece by piece and a new screen needs no code change to be
  translated.
- English is the source text itself and has no table.
- A line whose key is already in the table replaces it, so a correction is one line.

## Adding a language

Not done yet, and the shape is meant to make it small:

1. a variant in `os_ui::lang::Language` with its `name()` (the endonym, as the list shows it) and its `source()`
   (`include_str!` of `assets/lang/<tag>.txt`);
2. the table itself, translated from the English the code draws.

General ▸ Language's choices come from `prefs::LANGUAGE`, whose entries are the endonyms of `Language::ALL`, and
`App::apply_language` sets the language `os_ui::lang` draws in; neither needs a change for a new table.

## What is translated today

The settings window, the menu bar and its menus, the dialogs, the hints of the hint bar and status panel, the main
windows' labels (channel rack, mixer, browser, playlist, piano roll), the piano roll's tool windows, and the plugin
editors' labels. The macOS native menu bar is translated too (`menu_bar.rs`): a menu's entries are translated when it
opens, so they follow a language change at once; the bar's own titles are set when it is installed, so they follow on
the next start.
