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
//! use webonsive::{Caps, combobox, combobox::{ComboboxOptions, OptionGroup}};
//! let m = combobox(&Caps::all(), "q", "/langs", Default::default());
//!
//! let m = combobox(&Caps::all(), "q", "/langs", ComboboxOptions::default()
//!     .query("ru")
//!     .suggestions(&[OptionGroup::new("Systems", &["Rust", "Zig"]), OptionGroup::flat(&["Ruby"])])
//!     .results(&["Rust", "Ruby"])
//!     .selected(&["Zig"])
//!     .multi(true)
//!     .create("/langs/new")
//!     .label("Language")
//!     .placeholder("Type a language"));
//! let html = m.into_string();
//! assert!(html.contains("<optgroup label=\"Systems\">"));
//! assert!(html.contains("name=\"sel\" value=\"Zig\""), "the selection rides along with the next search");
//! assert!(html.contains("href=\"/langs?q=ru&amp;sel=Zig&amp;sel=Rust\""), "a result adds itself");
//! assert!(html.contains("href=\"/langs?q=ru\" aria-label=\"Remove Zig\""), "a chip removes itself");
//! ```

use maud::{Markup, html};

use crate::{Caps, state::encode};

/// A group of datalist suggestions; `label` `None` for ungrouped values.
#[derive(Clone, Copy, Debug)]
pub struct OptionGroup<'a> {
    label: Option<&'a str>,
    values: &'a [&'a str],
}

impl<'a> OptionGroup<'a> {
    /// A labelled `<optgroup>`.
    pub const fn new(label: &'a str, values: &'a [&'a str]) -> Self {
        OptionGroup { label: Some(label), values }
    }
    /// Ungrouped values.
    pub const fn flat(values: &'a [&'a str]) -> Self {
        OptionGroup { label: None, values }
    }
    /// The values in this group.
    pub const fn values(&self) -> &'a [&'a str] {
        self.values
    }
}

/// Options for [`combobox`]; `Default::default()` is an empty single-select search box.
#[derive(Clone, Copy, Debug)]
pub struct ComboboxOptions<'a> {
    /// The text currently in the input.
    pub query: &'a str,
    /// Datalist suggestions, grouped or flat.
    pub suggestions: &'a [OptionGroup<'a>],
    /// The server's results for `query`; each becomes a link that selects it.
    pub results: &'a [&'a str],
    /// The current selection, shown as chips and posted back as `sel` fields.
    pub selected: &'a [&'a str],
    /// Results add to the selection instead of replacing it.
    pub multi: bool,
    /// A post action shown as a "Create" row when `query` is set and nothing matches; it
    /// receives the text as `name` and the selection as `sel`.
    pub create: Option<&'a str>,
    /// Accessible name of the input.
    pub label: &'a str,
    /// Placeholder of the input.
    pub placeholder: &'a str,
}

impl Default for ComboboxOptions<'_> {
    fn default() -> Self {
        ComboboxOptions {
            query: "",
            suggestions: &[],
            results: &[],
            selected: &[],
            multi: false,
            create: None,
            label: "Search",
            placeholder: "Type to search\u{2026}",
        }
    }
}

impl<'a> ComboboxOptions<'a> {
    /// The text currently in the input.
    pub fn query(mut self, query: &'a str) -> Self {
        self.query = query;
        self
    }
    /// Datalist suggestions.
    pub fn suggestions(mut self, groups: &'a [OptionGroup<'a>]) -> Self {
        self.suggestions = groups;
        self
    }
    /// The server's results for the query.
    pub fn results(mut self, results: &'a [&'a str]) -> Self {
        self.results = results;
        self
    }
    /// The current selection.
    pub fn selected(mut self, selected: &'a [&'a str]) -> Self {
        self.selected = selected;
        self
    }
    /// Keep several values.
    pub fn multi(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }
    /// Post action for the "Create" row.
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

/// A search form sending `name` (the text) and `sel` (the selection) to `action` by GET.
pub fn combobox(_caps: &Caps, name: &str, action: &str, options: ComboboxOptions) -> Markup {
    let ComboboxOptions { query, suggestions, results, selected, multi, create, label, placeholder } = options;
    let list_id = format!("{name}-options");
    let results_id = format!("{name}-results");
    let link = |q: &str, sel: &[&str]| {
        let mut parts = vec![format!("{}={}", encode(name), encode(q))];
        parts.extend(sel.iter().map(|s| format!("sel={}", encode(s))));
        format!("{action}?{}", parts.join("&"))
    };
    let add = |v: &str| -> String {
        let mut sel: Vec<&str> = if multi { selected.to_vec() } else { Vec::new() };
        sel.push(v);
        link(query, &sel)
    };
    let remove = |v: &str| -> String {
        let sel: Vec<&str> = selected.iter().copied().filter(|s| *s != v).collect();
        link(query, &sel)
    };
    let nothing = results.is_empty() && !query.is_empty();
    html! {
        search class="wo-combobox" {
            form method="get" action=(action) {
                @if !selected.is_empty() {
                    ul class="wo-combobox-chips" aria-label="Selected" {
                        @for v in selected {
                            li class="wo-combobox-chip" {
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
                    @for g in suggestions {
                        @match g.label {
                            Some(l) => optgroup label=(l) { @for v in g.values { option value=(v) {} } },
                            None => { @for v in g.values { option value=(v) {} } },
                        }
                    }
                }
                button type="submit" class="wo-primary" { "Search" }
            }
            div id=(results_id) class="wo-combobox-results" aria-live="polite" {
                @if !results.is_empty() {
                    p class="wo-combobox-status" { (results.len()) @if results.len() == 1 { " match" } @else { " matches" } }
                    ul role="listbox" aria-label="Results" aria-multiselectable=[multi.then_some("true")] {
                        @for r in results {
                            @let picked = selected.contains(r);
                            li role="option" aria-selected=(picked) {
                                @if picked { (r) span class="wo-combobox-picked" { " selected" } }
                                @else { a href=(add(r)) { (r) } }
                            }
                        }
                    }
                } @else if nothing {
                    p class="wo-combobox-status" { "No matches." }
                    @if let Some(to) = create {
                        form method="post" action=(to) class="wo-combobox-create" {
                            @for v in selected { input type="hidden" name="sel" value=(v); }
                            input type="hidden" name="name" value=(query);
                            button type="submit" { "Create \u{201c}" (query) "\u{201d}" }
                        }
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-combobox form { display: flex; flex-wrap: wrap; gap: var(--wo-space); align-items: center; }
.wo-combobox input[type=search] { flex: 1; min-width: 10rem; }
.wo-combobox-chips { display: contents; }
.wo-combobox-chip {
  display: inline-flex; align-items: center; gap: 0.25rem; padding: 0.15rem 0.35rem 0.15rem 0.6rem;
  border: 1px solid var(--wo-line); border-radius: 1rem; background: var(--wo-surface); font-size: 0.875rem;
}
.wo-combobox-chip a { color: var(--wo-muted); text-decoration: none; padding: 0 0.3rem; border-radius: 1rem; line-height: 1.2; }
.wo-combobox-chip a:hover { color: var(--wo-fg); background: var(--wo-bg); }
.wo-combobox-results { margin: var(--wo-space) 0 calc(var(--wo-space) * 2); }
.wo-combobox-status { margin: 0 0 var(--wo-space); font-size: 0.875rem; color: var(--wo-muted); }
.wo-combobox-results [role=listbox] { list-style: none; margin: 0; padding: 0; border: 1px solid var(--wo-line); border-radius: var(--wo-radius); overflow: hidden; }
.wo-combobox-results [role=option] { max-width: none; }
.wo-combobox-results [role=option] + [role=option] { border-top: 1px solid var(--wo-line); }
.wo-combobox-results [role=option] a { display: block; padding: 0.5rem 0.75rem; text-decoration: none; color: inherit; }
.wo-combobox-results [role=option] a:hover, .wo-combobox-results [role=option] a:focus-visible { background: var(--wo-surface); outline-offset: -2px; }
.wo-combobox-results [role=option][aria-selected=true] { padding: 0.5rem 0.75rem; color: var(--wo-muted); }
.wo-combobox-picked { font-size: 0.875rem; }
.wo-combobox-create { display: inline-block; }
"#;
