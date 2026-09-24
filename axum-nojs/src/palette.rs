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
//! - `<search>` (Chrome 118, Firefox 118, Safari 17) around a GET `<form>`; the server
//!   redirects an exact name ([`Palette::exact`]) and lists the matches of anything else.
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
//! use axum_nojs::prelude::*;
//! // A search for "new" that matched no command exactly.
//! let ui = Ui::from_request("/search", "q=new", "");
//! let palette = ui.palette("/search")
//!     .group("Go to")
//!     .command("Open settings", "/settings")
//!     .command("New invoice", "/invoices/new").keywords("bill create");
//! assert_eq!(palette.exact(), None, "an exact name would redirect");
//! let m = palette.render().into_string();
//! assert!(m.contains(r#"popovertarget="palette""#) && m.contains(r#"list="palette-list""#));
//! assert!(m.contains("nojs-palette-results") && m.contains("New invoice"));
//! let ui = Ui::from_request("/search", "q=open+settings", "");
//! assert_eq!(ui.palette("/search").command("Open settings", "/settings").exact(), Some("/settings"));
//! ```

use maud::{Markup, Render, html};

use crate::{Cap, Ui};

/// One destination in the palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Command<'a> {
    label: &'a str,
    href: &'a str,
    group: &'a str,
    keywords: &'a str,
}

/// The command whose label is `query`, ignoring case and outer spaces: where Enter goes.
fn exact<'c, 'a>(commands: &'c [Command<'a>], query: &str) -> Option<&'c Command<'a>> {
    let q = query.trim();
    commands.iter().find(|c| c.label.eq_ignore_ascii_case(q))
}

/// Commands whose label or keywords contain every word of `query`, labels that start with it first.
fn matches<'c, 'a>(commands: &'c [Command<'a>], query: &str) -> Vec<&'c Command<'a>> {
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

/// A command palette submitting `q` to its action with GET, made by [`Ui::palette`]. The
/// request's `?q=` is the search; its results show below the opener.
#[derive(Clone, Debug)]
pub struct Palette<'a> {
    ui: &'a Ui,
    id: &'a str,
    action: &'a str,
    commands: Vec<Command<'a>>,
    group: &'a str,
    label: &'a str,
    key: char,
}

impl Ui {
    /// A palette `#palette` whose searches go to `action`; add destinations with
    /// [`Palette::command`].
    pub fn palette<'a>(&'a self, action: &'a str) -> Palette<'a> {
        Palette { ui: self, id: "palette", action, commands: Vec::new(), group: "", label: "Search", key: 'k' }
    }
}

impl<'a> Palette<'a> {
    /// A destination: what the visitor types or picks, and where it goes.
    pub fn command(mut self, label: &'a str, href: &'a str) -> Self {
        self.commands.push(Command { label, href, group: self.group, keywords: "" });
        self
    }

    /// Several `(label, href)` destinations at once.
    pub fn commands(self, commands: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        commands.into_iter().fold(self, |p, (label, href)| p.command(label, href))
    }

    /// Extra words that find the command added last, space-separated; never shown.
    pub fn keywords(mut self, keywords: &'a str) -> Self {
        if let Some(c) = self.commands.last_mut() {
            c.keywords = keywords;
        }
        self
    }

    /// List the commands added after this under a heading.
    pub fn group(mut self, heading: &'a str) -> Self {
        self.group = heading;
        self
    }

    /// The opener's label (default "Search").
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    /// The access key (default `k`).
    pub fn key(mut self, key: char) -> Self {
        self.key = key;
        self
    }

    /// The palette's id instead of `palette`.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = id;
        self
    }

    /// Where the request's search goes when it names a command exactly (ignoring case):
    /// the handler redirects there instead of rendering.
    pub fn exact(&self) -> Option<&'a str> {
        exact(&self.commands, self.ui.param("q")?).map(|c| c.href)
    }
}

impl Render for Palette<'_> {
    fn render(&self) -> Markup {
        let Palette { ui, id, action, ref commands, label, key, .. } = *self;
        let query = ui.param("q").filter(|q| !q.trim().is_empty());
        let list_id = format!("{id}-list");
        let shortcut = format!("Alt+Shift+{}", key.to_ascii_uppercase());
        let key = key.to_string();
        let hint = html! { kbd class="nojs-palette-kbd" { (shortcut) } };
        let popover = ui.has(Cap::Popover);
        let results = query.map(|q| {
            let found = matches(commands, q);
            html! {
                section class="nojs-palette-results" aria-labelledby={ (id) "-results" } {
                    h2 id={ (id) "-results" } { (found.len()) @if found.len() == 1 { " match" } @else { " matches" } " for \u{201c}" (q) "\u{201d}" }
                    @if found.is_empty() { p { "Nothing by that name. Try one word, or pick from the list." } }
                    @else { (grouped(found)) }
                }
            }
        });
        let form = html! {
            search {
                form method="get" action=(action) class="nojs-palette-form" {
                    input type="search" name="q" list=(list_id) autofocus autocomplete="off"
                        placeholder="Type a command or a page" aria-label=(label) value=[query];
                    button type="submit" class="nojs-primary" { "Go" }
                }
            }
            datalist id=(list_id) { @for c in commands { option value=(c.label) {} } }
        };
        html! {
            div class="nojs-palette" {
                @if popover {
                    button type="button" class="nojs-palette-open" popovertarget=(id) accesskey=(key) aria-keyshortcuts=(shortcut) {
                        (label) " " (hint)
                    }
                    div id=(id) class="nojs-palette-panel" popover { (form) (grouped(commands.iter().collect())) }
                    // A popover cannot arrive open, so the results of a search sit in the page.
                    @if let Some(r) = results { (r) }
                } @else {
                    details class="nojs-palette-details" id=(id) open[query.is_some()] {
                        summary class="nojs-palette-open" accesskey=(key) aria-keyshortcuts=(shortcut) { (label) " " (hint) }
                        div class="nojs-palette-panel" {
                            (form)
                            @if let Some(r) = results { (r) } @else { (grouped(commands.iter().collect())) }
                        }
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
            div class="nojs-palette-group" {
                @if !g.is_empty() { p class="nojs-palette-heading" { (g) } }
                ul { @for c in commands.iter().filter(|c| c.group == g) { li { a href=(c.href) { (c.label) } } } }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-palette-open {
  display: inline-flex; align-items: center; gap: var(--nojs-space); min-width: 16rem; justify-content: space-between;
  padding: 0.5rem 0.75rem; color: var(--nojs-muted); background: var(--nojs-surface); cursor: pointer; font: inherit;
  border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); list-style: none;
}
.nojs-palette-open::-webkit-details-marker { display: none; }
.nojs-palette-kbd { font-size: 0.75rem; padding: 0.1rem 0.4rem; border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); }
.nojs-palette-panel {
  box-sizing: border-box; width: min(36rem, calc(100vw - 2rem)); padding: calc(var(--nojs-space) * 2);
  color: var(--nojs-fg); background: var(--nojs-surface); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius);
  box-shadow: 0 16px 48px color-mix(in srgb, var(--nojs-fg) 20%, transparent);
}
.nojs-palette-panel[popover] { margin: 12vh auto auto; max-height: 70vh; overflow: auto; }
.nojs-palette-panel[popover]::backdrop { background: color-mix(in srgb, var(--nojs-fg) 25%, transparent); }
.nojs-palette-details .nojs-palette-panel { margin-top: var(--nojs-space); }
.nojs-palette-form { display: flex; gap: var(--nojs-space); margin-bottom: var(--nojs-space); }
.nojs-palette-form input { flex: 1; font-size: 1.125rem; padding: 0.5rem 0.75rem; }
.nojs-palette-heading { margin: calc(var(--nojs-space) * 1.5) 0 0.25rem; font-size: 0.8rem; color: var(--nojs-muted); }
.nojs-palette ul { list-style: none; margin: 0; padding: 0; }
.nojs-palette li a { display: block; padding: 0.4rem 0.75rem; border-radius: var(--nojs-radius); color: var(--nojs-fg); text-decoration: none; }
.nojs-palette li a:hover, .nojs-palette li a:focus-visible { background: var(--nojs-bg); }
.nojs-palette-results { margin-top: calc(var(--nojs-space) * 3); }
.nojs-palette-results h2 { font-size: 1.125rem; }
"#;
