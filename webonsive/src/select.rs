//! # Select
//!
//! A `<select>` whose closed state shows the chosen option's full content, no script. Options
//! come in labelled groups and can carry an icon; a long list gets a filter box.
//!
//! **Platform features:**
//! - `<select>` with a `<button>` first child holding `<selectedcontent>` (Chrome 135, Safari
//!   27; Firefox behind flags): the button is the closed control and `<selectedcontent>`
//!   mirrors the picked option's markup into it, so options can carry a swatch or an icon.
//! - `appearance: base-select` on the select and its `::picker(select)` pseudo-element hands
//!   both to author CSS.
//! - `<optgroup label>` (baseline) for [`Group`]s.
//! - A filter box when there are more than [`SelectOptions::search_over`] options: an
//!   `<input type="search" name="<name>-q">` and a button with `formmethod="get"` and
//!   `formaction` (baseline 2015), so filtering re-requests the page through the enclosing
//!   form without saving it. The server renders only the options whose text contains the
//!   query, and always the selected one.
//!
//! **What it does not do without script:** type-ahead search beyond what the browser offers;
//! long lists get a server-side filter box instead.
//!
//! **Fallback:** without `Caps::BaseSelect` a plain `<select>` with plain options (icons as
//! text before the label). Older parsers also drop a `<button>` inside `<select>`, so the
//! enhanced markup is only emitted when the browser is known to want it.
//!
//! **Enhanced:** the enhancement script filters as you type, through the same GET.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, select, select::{Group, SelectOption, SelectOptions}};
//! let sizes = [SelectOption::new("s", "Small"), SelectOption::new("l", "Large").icon("🐘")];
//! let m = select(&Caps::all(), "size", &[Group::flat(&sizes)], "l", Default::default());
//! assert!(m.into_string().contains("<selectedcontent>"));
//!
//! let fruit = [SelectOption::new("apple", "Apple"), SelectOption::new("kiwi", "Kiwi").content(html! { b { "Kiwi" } })];
//! let veg = [SelectOption::new("leek", "Leek")];
//! let m = select(&Caps::all(), "food", &[Group::new("Fruit", &fruit), Group::new("Vegetables", &veg)], "leek",
//!                SelectOptions::default().search("/shop", "k").search_over(2)).into_string();
//! assert!(m.contains("<optgroup label=\"Fruit\">") && !m.contains("Apple"), "filtered to 'k'");
//! assert!(m.contains("formmethod=\"get\" formaction=\"/shop\""));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps};

/// One option: its value, its text (what the filter matches), an optional icon and optional
/// rich content shown instead of the text.
#[derive(Clone, Debug)]
pub struct SelectOption<'a> {
    /// Posted value.
    pub value: &'a str,
    /// Plain label; matched by the filter.
    pub text: &'a str,
    /// A short icon (an emoji or a glyph) before the label, hidden from assistive tech.
    pub icon: Option<&'a str>,
    /// Markup shown instead of `text` when the select is rich.
    pub content: Option<Markup>,
}

impl<'a> SelectOption<'a> {
    /// An option with a plain label.
    pub fn new(value: &'a str, text: &'a str) -> Self {
        SelectOption { value, text, icon: None, content: None }
    }
    /// An icon before the label.
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Rich markup instead of the text (with `Caps::BaseSelect` only).
    pub fn content(mut self, content: Markup) -> Self {
        self.content = Some(content);
        self
    }
}

/// Options under an `<optgroup>`, or loose.
#[derive(Clone, Copy, Debug)]
pub struct Group<'a> {
    /// The `<optgroup label>`; `None` for loose options.
    pub label: Option<&'a str>,
    /// The options.
    pub options: &'a [SelectOption<'a>],
}

impl<'a> Group<'a> {
    /// A labelled group.
    pub fn new(label: &'a str, options: &'a [SelectOption<'a>]) -> Self {
        Group { label: Some(label), options }
    }
    /// Options with no group.
    pub fn flat(options: &'a [SelectOption<'a>]) -> Self {
        Group { label: None, options }
    }
}

/// Options for [`select`]; `Default::default()` has no filter box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectOptions<'a> {
    /// `(formaction, query)`: where the filter's GET goes and the current `<name>-q`.
    pub search: Option<(&'a str, &'a str)>,
    /// Show the filter box above this many options.
    pub search_over: usize,
}

impl Default for SelectOptions<'_> {
    fn default() -> Self {
        SelectOptions { search: None, search_over: 15 }
    }
}

impl<'a> SelectOptions<'a> {
    /// A filter box submitting to `action` with GET; `query` is the current `<name>-q`.
    pub fn search(mut self, action: &'a str, query: &'a str) -> Self {
        self.search = Some((action, query));
        self
    }
    /// Show the filter box above this many options (default 15).
    pub fn search_over(mut self, n: usize) -> Self {
        self.search_over = n;
        self
    }
}

/// `groups` of options; `selected` is the current value from the server.
pub fn select(caps: &Caps, name: &str, groups: &[Group], selected: &str, options: SelectOptions) -> Markup {
    let SelectOptions { search, search_over } = options;
    let rich = caps.has(Cap::BaseSelect);
    let total: usize = groups.iter().map(|g| g.options.len()).sum();
    let search = search.filter(|_| total > search_over);
    let query = search.map(|(_, q)| q.trim().to_lowercase()).unwrap_or_default();
    let shown = |o: &SelectOption| query.is_empty() || o.value == selected || o.text.to_lowercase().contains(&query);
    let option = |o: &SelectOption| html! {
        option value=(o.value) selected[o.value == selected] {
            @if let Some(i) = o.icon { span class="wo-select-icon" aria-hidden="true" { (i) } " " }
            @match (&o.content, rich) { (Some(c), true) => (c), _ => (o.text) }
        }
    };
    html! {
        span class="wo-select" {
            @if let Some((action, q)) = search {
                span class="wo-select-search" {
                    input type="search" name={ (name) "-q" } value=(q) placeholder="Filter" aria-label="Filter options";
                    button type="submit" formmethod="get" formaction=(action) formnovalidate { "Filter" }
                }
            }
            select id=(name) name=(name) {
                @if rich { button type="button" { selectedcontent {} } }
                @for g in groups {
                    @let visible: Vec<&SelectOption> = g.options.iter().filter(|o| shown(o)).collect();
                    @if let (Some(label), false) = (g.label, visible.is_empty()) {
                        optgroup label=(label) { @for o in &visible { (option(o)) } }
                    } @else {
                        @for o in &visible { (option(o)) }
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-select { display: inline-grid; gap: 0.4rem; }
.wo-select-search { display: flex; gap: 0.4rem; }
.wo-select-search input { flex: 1; min-width: 0; }
.wo-select select, .wo-select select::picker(select) { appearance: base-select; }
.wo-select select { min-width: 12rem; }
.wo-select select::picker(select) {
  border: 1px solid var(--wo-line); border-radius: var(--wo-radius); padding: var(--wo-space) 0;
  background: var(--wo-surface); color: var(--wo-fg); box-shadow: 0 8px 24px color-mix(in srgb, var(--wo-fg) 14%, transparent);
  max-height: 20rem;
}
.wo-select option { padding: 0.4rem 1rem; }
.wo-select option:hover, .wo-select option:checked { background: var(--wo-bg); }
.wo-select option::checkmark { order: 1; margin-left: auto; }
.wo-select optgroup { font-weight: 600; color: var(--wo-muted); padding: 0.25rem 0; }
.wo-select optgroup option { font-weight: 400; color: var(--wo-fg); }
.wo-select-icon { display: inline-block; width: 1.25em; text-align: center; }
.wo-swatch { display: inline-block; width: 1em; height: 1em; border-radius: 50%; vertical-align: -0.15em; margin-right: 0.4em; border: 1px solid var(--wo-line); }
"#;
