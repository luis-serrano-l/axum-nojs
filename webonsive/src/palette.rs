//! # Command palette
//!
//! A search box that jumps anywhere: open it from a button or with an access key, type, pick
//! a suggestion, press Enter. The browser filters the suggestions as you type; the server does
//! the rest. An exact name goes straight to its page; anything else lands on a results list.
//! This is the flagship "no script needed" component.
//!
//! **Platform features:**
//! - `popover` (Chrome 114, Firefox 125, Safari 17) opened by `popovertarget`; light dismiss
//!   and Escape for free; `autofocus` puts the caret in the box when it opens.
//! - `<datalist>` bound by `list=` for as-you-type suggestions, filtered by the browser.
//! - `<search>` (Chrome 118, Firefox 118, Safari 17) around a GET `<form>`; [`exact`] and
//!   [`matches()`] do the server's part.
//! - `accesskey` on the opener (Alt+Shift+K in Chrome and Firefox, Ctrl+Option+K in Safari),
//!   announced with `aria-keyshortcuts`.
//!
//! **Fallback:** without `Caps` `Popover` the palette is a `<details>` disclosure with the
//! same form inside. A browser without `<datalist>` shows a plain search box; the results
//! page still works.
//!
//! **What it does not do without script:** arrow-key navigation through a live results list
//! and the global Ctrl+K shortcut (the access key stands in for it).
//!
//! ```rust
//! use webonsive::{Caps, command_palette, palette::{Command, PaletteOptions, exact, matches}};
//! const CMDS: &[Command] = &[
//!     Command::new("Open settings", "/settings").group("Go to"),
//!     Command::new("New invoice", "/invoices/new").keywords("bill create"),
//! ];
//! let m = command_palette(&Caps::all(), "cmd", "/search", CMDS, Default::default()).into_string();
//! assert!(m.contains(r#"popovertarget="cmd""#) && m.contains(r#"list="cmd-list""#));
//! assert_eq!(exact(CMDS, "open settings").map(|c| c.href), Some("/settings"));
//! assert_eq!(matches(CMDS, "bill").len(), 1);
//! let m = command_palette(&Caps::all(), "cmd", "/search", CMDS, PaletteOptions::default().query("new")).into_string();
//! assert!(m.contains("wo-palette-results") && m.contains("New invoice"));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps};

/// One destination in the palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command<'a> {
    /// What the user types or picks.
    pub label: &'a str,
    /// Where it goes.
    pub href: &'a str,
    /// A heading it is listed under.
    pub group: &'a str,
    /// Extra words that find it, space-separated; never shown.
    pub keywords: &'a str,
}

impl<'a> Command<'a> {
    /// A command `label` going to `href`.
    pub const fn new(label: &'a str, href: &'a str) -> Self {
        Command { label, href, group: "", keywords: "" }
    }
    /// List it under `group`.
    pub const fn group(mut self, group: &'a str) -> Self {
        self.group = group;
        self
    }
    /// Extra words that find it.
    pub const fn keywords(mut self, keywords: &'a str) -> Self {
        self.keywords = keywords;
        self
    }
}

/// The command whose label is `query`, ignoring case and outer spaces: where Enter goes.
pub fn exact<'c, 'a>(commands: &'c [Command<'a>], query: &str) -> Option<&'c Command<'a>> {
    let q = query.trim();
    commands.iter().find(|c| c.label.eq_ignore_ascii_case(q))
}

/// Commands whose label or keywords contain every word of `query`, labels that start with it first.
pub fn matches<'c, 'a>(commands: &'c [Command<'a>], query: &str) -> Vec<&'c Command<'a>> {
    let q = query.trim().to_lowercase();
    let words: Vec<&str> = q.split_whitespace().collect();
    let mut found: Vec<&Command> = commands
        .iter()
        .filter(|c| {
            let hay = format!("{} {}", c.label, c.keywords).to_lowercase();
            words.iter().all(|w| hay.contains(w))
        })
        .collect();
    found.sort_by_key(|c| !c.label.to_lowercase().starts_with(&q));
    found
}

/// Options for [`command_palette`].
#[derive(Clone, Debug)]
pub struct PaletteOptions<'a> {
    /// The opener's label (default "Search").
    pub label: &'a str,
    /// The access key (default `k`).
    pub key: char,
    /// A search that found no exact command: shown in the box and as a results list.
    pub query: Option<&'a str>,
}

impl Default for PaletteOptions<'_> {
    fn default() -> Self {
        PaletteOptions { label: "Search", key: 'k', query: None }
    }
}

impl<'a> PaletteOptions<'a> {
    /// The opener's label.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
    /// The access key.
    pub fn key(mut self, key: char) -> Self {
        self.key = key;
        self
    }
    /// Show the results for `query` below the opener.
    pub fn query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }
}

/// A command palette `id` over `commands`, submitting `q` to `action` with GET.
pub fn command_palette(caps: &Caps, id: &str, action: &str, commands: &[Command], options: PaletteOptions) -> Markup {
    let list_id = format!("{id}-list");
    let shortcut = format!("Alt+Shift+{}", options.key.to_ascii_uppercase());
    let key = options.key.to_string();
    let hint = html! { kbd class="wo-palette-kbd" { (shortcut) } };
    let popover = caps.has(Cap::Popover);
    let results = options.query.map(|q| {
        let found = matches(commands, q);
        html! {
            section class="wo-palette-results" aria-labelledby={ (id) "-results" } {
                h2 id={ (id) "-results" } { (found.len()) @if found.len() == 1 { " match" } @else { " matches" } " for \u{201c}" (q) "\u{201d}" }
                @if found.is_empty() { p { "Nothing by that name. Try one word, or pick from the list." } }
                @else { (grouped(found)) }
            }
        }
    });
    let form = html! {
        search {
            form method="get" action=(action) class="wo-palette-form" {
                input type="search" name="q" list=(list_id) autofocus autocomplete="off"
                    placeholder="Type a command or a page" aria-label=(options.label) value=[options.query];
                button type="submit" class="wo-primary" { "Go" }
            }
        }
        datalist id=(list_id) { @for c in commands { option value=(c.label) {} } }
    };
    html! {
        div class="wo-palette" {
            @if popover {
                button type="button" class="wo-palette-open" popovertarget=(id) accesskey=(key) aria-keyshortcuts=(shortcut) {
                    (options.label) " " (hint)
                }
                div id=(id) class="wo-palette-panel" popover { (form) (grouped(commands.iter().collect())) }
                // A popover cannot arrive open, so the results of a search sit in the page.
                @if let Some(r) = results { (r) }
            } @else {
                details class="wo-palette-details" id=(id) open[options.query.is_some()] {
                    summary class="wo-palette-open" accesskey=(key) aria-keyshortcuts=(shortcut) { (options.label) " " (hint) }
                    div class="wo-palette-panel" {
                        (form)
                        @if let Some(r) = results { (r) } @else { (grouped(commands.iter().collect())) }
                    }
                }
            }
        }
    }
}

fn grouped(commands: Vec<&Command>) -> Markup {
    let mut groups: Vec<&str> = Vec::new();
    for c in &commands {
        if !groups.contains(&c.group) {
            groups.push(c.group);
        }
    }
    html! {
        @for g in groups {
            div class="wo-palette-group" {
                @if !g.is_empty() { p class="wo-palette-heading" { (g) } }
                ul { @for c in commands.iter().filter(|c| c.group == g) { li { a href=(c.href) { (c.label) } } } }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-palette-open {
  display: inline-flex; align-items: center; gap: var(--wo-space); min-width: 16rem; justify-content: space-between;
  padding: 0.5rem 0.75rem; color: var(--wo-muted); background: var(--wo-surface); cursor: pointer; font: inherit;
  border: 1px solid var(--wo-line); border-radius: var(--wo-radius); list-style: none;
}
.wo-palette-open::-webkit-details-marker { display: none; }
.wo-palette-kbd { font-size: 0.75rem; padding: 0.1rem 0.4rem; border: 1px solid var(--wo-line); border-radius: var(--wo-radius); }
.wo-palette-panel {
  box-sizing: border-box; width: min(36rem, calc(100vw - 2rem)); padding: calc(var(--wo-space) * 2);
  color: var(--wo-fg); background: var(--wo-surface); border: 1px solid var(--wo-line); border-radius: var(--wo-radius);
  box-shadow: 0 16px 48px color-mix(in srgb, var(--wo-fg) 20%, transparent);
}
.wo-palette-panel[popover] { margin: 12vh auto auto; max-height: 70vh; overflow: auto; }
.wo-palette-panel[popover]::backdrop { background: color-mix(in srgb, var(--wo-fg) 25%, transparent); }
.wo-palette-details .wo-palette-panel { margin-top: var(--wo-space); }
.wo-palette-form { display: flex; gap: var(--wo-space); margin-bottom: var(--wo-space); }
.wo-palette-form input { flex: 1; font-size: 1.125rem; padding: 0.5rem 0.75rem; }
.wo-palette-heading { margin: calc(var(--wo-space) * 1.5) 0 0.25rem; font-size: 0.8rem; color: var(--wo-muted); }
.wo-palette ul { list-style: none; margin: 0; padding: 0; }
.wo-palette li a { display: block; padding: 0.4rem 0.75rem; border-radius: var(--wo-radius); color: var(--wo-fg); text-decoration: none; }
.wo-palette li a:hover, .wo-palette li a:focus-visible { background: var(--wo-bg); }
.wo-palette-results { margin-top: calc(var(--wo-space) * 3); }
.wo-palette-results h2 { font-size: 1.125rem; }
"#;
