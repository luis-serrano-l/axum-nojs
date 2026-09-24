//! # Combobox
//!
//! A text input with native suggestions, a server-filtered result list, and a selection
//! shown as removable chips, no script. Suggestions can be grouped, several values can be
//! selected, a "create" row appears when nothing matches, and the selection survives every
//! re-filter because it travels in the form as hidden fields.
//!
//! **Platform features:**
//! - `<input list>` + `<datalist>` (baseline 2020) gives native type-ahead from a fixed list
//!   the server already knows; `<optgroup>` inside it (Chrome 20, Firefox 4, Safari 12.1)
//!   groups the suggestions where the browser draws them grouped.
//! - `<search>` element (baseline 2023) for semantics; `role="listbox"` / `role="option"` on
//!   the result list and `aria-live="polite"` on the results, so a swapped result list is
//!   announced.
//! - Submitting the form (Enter or the button) re-renders with the server's results. Every
//!   result is a link that adds it to the selection (or replaces it, unless `multi`), every
//!   chip has a link that removes it, and the "create" row is a plain post form.
//!
//! **What it does not do without script:** filter as you type against the server or move
//! through results with arrow keys and a live `aria-activedescendant`; suggestions come from
//! the `<datalist>` sent with the page.
//!
//! **Fallback:** none needed; `Caps` is accepted for uniformity and unused.
//!
//! **Without script:** results update per round trip and the arrow keys do not move into
//! the list (Tab does). With the enhancement script the form is submitted as you type and
//! ArrowDown/ArrowUp walk the input and the results.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! // The text is `?q=` and the selection `?sel=`, read from the request.
//! let ui = Ui::from_request("/langs", "q=ru&sel=Zig", "");
//! let m = ui.combobox("q", "/langs")
//!     .group("Systems", ["Rust", "Zig"])
//!     .options(["Ruby"])
//!     .multi()
//!     .create("/langs/new")
//!     .label("Language")
//!     .placeholder("Type a language");
//! let html = m.render().into_string();
//! assert!(html.contains("<optgroup label=\"Systems\">"));
//! assert!(html.contains("name=\"sel\" value=\"Zig\""), "the selection rides along with the next search");
//! assert!(html.contains("href=\"/langs?q=ru&amp;sel=Zig&amp;sel=Rust\""), "a result adds itself");
//! assert!(html.contains("href=\"/langs?q=ru\" aria-label=\"Remove Zig\""), "a chip removes itself");
//! ```

use maud::{Markup, Render, html};

use crate::{Ui, state::encode};

/// A search form sending `name` (the text) and `sel` (the selection) by GET, made by
/// [`Ui::combobox`]. Single choice, labelled "Search", unless told otherwise.
#[derive(Clone, Debug)]
pub struct Combobox<'a> {
    ui: &'a Ui,
    name: &'a str,
    action: &'a str,
    suggestions: Vec<(Option<&'a str>, Vec<&'a str>)>,
    results: Option<Vec<&'a str>>,
    multi: bool,
    create: Option<&'a str>,
    label: &'a str,
    placeholder: &'a str,
}

impl Ui {
    /// A combobox whose text is the query parameter `name` and whose selection is every
    /// `sel`, both read from this request; the form goes to `action`.
    pub fn combobox<'a>(&'a self, name: &'a str, action: &'a str) -> Combobox<'a> {
        Combobox {
            ui: self,
            name,
            action,
            suggestions: Vec::new(),
            results: None,
            multi: false,
            create: None,
            label: "Search",
            placeholder: "Type to search\u{2026}",
        }
    }
}

impl<'a> Combobox<'a> {
    /// Suggestions with no `<optgroup>`.
    pub fn options(mut self, values: impl IntoIterator<Item = &'a str>) -> Self {
        self.suggestions.push((None, values.into_iter().collect()));
        self
    }

    /// Suggestions under `<optgroup label>`.
    pub fn group(mut self, label: &'a str, values: impl IntoIterator<Item = &'a str>) -> Self {
        self.suggestions.push((Some(label), values.into_iter().collect()));
        self
    }

    /// The server's own results for the query. Without this, the results are the
    /// suggestions containing the query, ignoring case.
    pub fn results(mut self, results: impl IntoIterator<Item = &'a str>) -> Self {
        self.results = Some(results.into_iter().collect());
        self
    }

    /// Results add to the selection instead of replacing it.
    pub fn multi(mut self) -> Self {
        self.multi = true;
        self
    }

    /// A "Create" row posting to `action` when the query matches nothing; it receives the
    /// text as `name` and the selection as `sel`.
    pub fn create(mut self, action: &'a str) -> Self {
        self.create = Some(action);
        self
    }

    /// Accessible name of the input.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    /// Placeholder of the input.
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
}

impl Render for Combobox<'_> {
    fn render(&self) -> Markup {
        let Combobox { ui, name, action, ref suggestions, ref results, multi, create, label, placeholder } = *self;
        let query = ui.param(name).unwrap_or("");
        let selected: Vec<&str> = ui.params("sel").collect();
        let needle = query.trim().to_lowercase();
        let results: Vec<&str> = match results {
            Some(r) => r.clone(),
            None if needle.is_empty() => Vec::new(),
            None => suggestions.iter().flat_map(|(_, v)| v.iter().copied()).filter(|v| v.to_lowercase().contains(&needle)).collect(),
        };
        let list_id = format!("{name}-options");
        let results_id = format!("{name}-results");
        let link = |q: &str, sel: &[&str]| {
            let mut parts = vec![format!("{}={}", encode(name), encode(q))];
            parts.extend(sel.iter().map(|s| format!("sel={}", encode(s))));
            format!("{action}?{}", parts.join("&"))
        };
        let add = |v: &str| -> String {
            let mut sel: Vec<&str> = if multi { selected.clone() } else { Vec::new() };
            sel.push(v);
            link(query, &sel)
        };
        let remove = |v: &str| -> String {
            let sel: Vec<&str> = selected.iter().copied().filter(|s| *s != v).collect();
            link(query, &sel)
        };
        let nothing = results.is_empty() && !query.is_empty();
        html! {
            search class="nojs-combobox" {
                form method="get" action=(action) {
                    @if !selected.is_empty() {
                        ul class="nojs-combobox-chips" aria-label="Selected" {
                            @for v in &selected {
                                li class="nojs-combobox-chip" {
                                    (v)
                                    input type="hidden" name="sel" value=(v);
                                    a href=(remove(v)) aria-label={ "Remove " (v) } { "\u{d7}" }
                                }
                            }
                        }
                    }
                    input type="search" name=(name) list=(list_id) value=(query) aria-label=(label)
                        aria-controls=(results_id) placeholder=(placeholder) autocomplete="off";
                    datalist id=(list_id) {
                        @for (group, values) in suggestions {
                            @match group {
                                Some(l) => optgroup label=(l) { @for v in values { option value=(v) {} } },
                                None => { @for v in values { option value=(v) {} } },
                            }
                        }
                    }
                    button type="submit" class="nojs-primary" { "Search" }
                }
                div id=(results_id) class="nojs-combobox-results" aria-live="polite" {
                    @if !results.is_empty() {
                        p class="nojs-combobox-status" { (results.len()) @if results.len() == 1 { " match" } @else { " matches" } }
                        ul role="listbox" aria-label="Results" aria-multiselectable=[multi.then_some("true")] {
                            @for r in &results {
                                @let picked = selected.contains(r);
                                li role="option" aria-selected=(picked) {
                                    @if picked { (r) span class="nojs-combobox-picked" { " selected" } }
                                    @else { a href=(add(r)) { (r) } }
                                }
                            }
                        }
                    } @else if nothing {
                        p class="nojs-combobox-status" { "No matches." }
                        @if let Some(to) = create {
                            form method="post" action=(to) class="nojs-combobox-create" {
                                @for v in &selected { input type="hidden" name="sel" value=(v); }
                                input type="hidden" name="name" value=(query);
                                button type="submit" { "Create \u{201c}" (query) "\u{201d}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-combobox form { display: flex; flex-wrap: wrap; gap: var(--nojs-space); align-items: center; }
.nojs-combobox input[type=search] { flex: 1; min-width: 10rem; }
.nojs-combobox-chips { display: contents; }
.nojs-combobox-chip {
  display: inline-flex; align-items: center; gap: 0.25rem; padding: 0.15rem 0.35rem 0.15rem 0.6rem;
  border: 1px solid var(--nojs-line); border-radius: 1rem; background: var(--nojs-surface); font-size: 0.875rem;
}
.nojs-combobox-chip a { color: var(--nojs-muted); text-decoration: none; padding: 0 0.3rem; border-radius: 1rem; line-height: 1.2; }
.nojs-combobox-chip a:hover { color: var(--nojs-fg); background: var(--nojs-bg); }
.nojs-combobox-results { margin: var(--nojs-space) 0 calc(var(--nojs-space) * 2); }
.nojs-combobox-status { margin: 0 0 var(--nojs-space); font-size: 0.875rem; color: var(--nojs-muted); }
.nojs-combobox-results [role=listbox] { list-style: none; margin: 0; padding: 0; border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); overflow: hidden; }
.nojs-combobox-results [role=option] { max-width: none; }
.nojs-combobox-results [role=option] + [role=option] { border-top: 1px solid var(--nojs-line); }
.nojs-combobox-results [role=option] a { display: block; padding: 0.5rem 0.75rem; text-decoration: none; color: inherit; }
.nojs-combobox-results [role=option] a:hover, .nojs-combobox-results [role=option] a:focus-visible { background: var(--nojs-surface); outline-offset: -2px; }
.nojs-combobox-results [role=option][aria-selected=true] { padding: 0.5rem 0.75rem; color: var(--nojs-muted); }
.nojs-combobox-picked { font-size: 0.875rem; }
.nojs-combobox-create { display: inline-block; }
"#;
