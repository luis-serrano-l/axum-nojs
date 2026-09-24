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
//! - `<optgroup label>` (baseline) for [`Select::group`].
//! - A filter box when there are more than [`Select::search_over`] options: an
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
//! use axum_nojs::{prelude::*, select::SelectOption};
//! let mut ui = Ui::from_request("/shop", "food-q=k", "");
//! ui.caps = Caps::all();
//! // Plain tuples of (value, label) or (value, label, icon) are options.
//! let m = ui.select("size", "l").options([("s", "Small", "🐭"), ("l", "Large", "🐘")]).label("Size");
//! let m = m.render().into_string();
//! assert!(m.contains("<selectedcontent>") && m.contains(r#"<label for="size">Size</label>"#));
//!
//! let m = ui.select("food", "leek")
//!     .group("Fruit", [SelectOption::new("apple", "Apple"), SelectOption::new("kiwi", "Kiwi").content(html! { b { "Kiwi" } })])
//!     .group("Vegetables", [("leek", "Leek")])
//!     .search("/shop")
//!     .search_over(2);
//! let m = m.render().into_string();
//! assert!(m.contains("<optgroup label=\"Fruit\">") && !m.contains("Apple"), "filtered to 'k'");
//! assert!(m.contains("formmethod=\"get\" formaction=\"/shop\""));
//! ```

use maud::{Markup, Render, html};

use crate::input::Input;
use crate::{Cap, Ui};

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
    pub const fn new(value: &'a str, text: &'a str) -> Self {
        SelectOption {
            value,
            text,
            icon: None,
            content: None,
        }
    }
    /// An icon before the label.
    pub const fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Rich markup instead of the text (with `Caps::BaseSelect` only).
    pub fn content(mut self, content: Markup) -> Self {
        self.content = Some(content);
        self
    }
}

/// `("m", "Medium")`: a value and its label.
impl<'a> From<(&'a str, &'a str)> for SelectOption<'a> {
    fn from((value, text): (&'a str, &'a str)) -> Self {
        SelectOption::new(value, text)
    }
}

/// `("m", "Medium", "🐕")`: a value, its label and an icon.
impl<'a> From<(&'a str, &'a str, &'a str)> for SelectOption<'a> {
    fn from((value, text, icon): (&'a str, &'a str, &'a str)) -> Self {
        SelectOption::new(value, text).icon(icon)
    }
}

/// Options under an `<optgroup label>`, or loose when `label` is `None`.
#[derive(Clone, Debug)]
struct Group<'a> {
    label: Option<&'a str>,
    options: Vec<SelectOption<'a>>,
}

/// A select, made by [`Ui::select`]. No filter box unless [`Select::search`] asks for one.
#[derive(Clone, Debug)]
pub struct Select<'a> {
    ui: &'a Ui,
    name: &'a str,
    selected: &'a str,
    groups: Vec<Group<'a>>,
    search: Option<&'a str>,
    search_over: usize,
    label: Option<&'a str>,
}

impl Ui {
    /// A select named `name` with `selected` chosen; add options with [`Select::options`] or
    /// [`Select::group`].
    pub fn select<'a>(&'a self, name: &'a str, selected: &'a str) -> Select<'a> {
        Select {
            ui: self,
            name,
            selected,
            groups: Vec::new(),
            search: None,
            search_over: 15,
            label: None,
        }
    }
}

impl<'a> Select<'a> {
    /// Options with no `<optgroup>`: [`SelectOption`]s or `(value, label)` and
    /// `(value, label, icon)` tuples.
    pub fn options<O: Into<SelectOption<'a>>>(
        mut self,
        options: impl IntoIterator<Item = O>,
    ) -> Self {
        self.groups.push(Group {
            label: None,
            options: options.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// Options under `<optgroup label>`.
    pub fn group<O: Into<SelectOption<'a>>>(
        mut self,
        label: &'a str,
        options: impl IntoIterator<Item = O>,
    ) -> Self {
        self.groups.push(Group {
            label: Some(label),
            options: options.into_iter().map(Into::into).collect(),
        });
        self
    }

    /// Several labelled groups at once: `(label, options)` pairs.
    pub fn groups<O: Into<SelectOption<'a>>, G: IntoIterator<Item = O>>(
        self,
        groups: impl IntoIterator<Item = (&'a str, G)>,
    ) -> Self {
        groups
            .into_iter()
            .fold(self, |s, (label, options)| s.group(label, options))
    }

    /// A filter box submitting `<name>-q` to `action` with GET; the options shown are those
    /// matching the `<name>-q` in this request's query, and always the selected one.
    pub fn search(mut self, action: &'a str) -> Self {
        self.search = Some(action);
        self
    }

    /// Show the filter box above this many options (default 15).
    pub fn search_over(mut self, n: usize) -> Self {
        self.search_over = n;
        self
    }

    /// A `<label>` above the select, in a `div.nojs-field` like a form field.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl Render for Select<'_> {
    fn render(&self) -> Markup {
        let Select {
            ui,
            name,
            selected,
            ref groups,
            search,
            search_over,
            label,
        } = *self;
        let rich = ui.has(Cap::BaseSelect);
        let total: usize = groups.iter().map(|g| g.options.len()).sum();
        let search = search.filter(|_| total > search_over);
        let q_name = format!("{name}-q");
        let q_id = format!("{name}-filter");
        let q = ui.param(&q_name).unwrap_or("");
        let query = if search.is_some() {
            q.trim().to_lowercase()
        } else {
            String::new()
        };
        let shown = |o: &SelectOption| {
            query.is_empty() || o.value == selected || o.text.to_lowercase().contains(&query)
        };
        let option = |o: &SelectOption| {
            html! {
                option value=(o.value) selected[o.value == selected] {
                    @if let Some(i) = o.icon { span class="nojs-select-icon" aria-hidden="true" { (i) } " " }
                    @match (&o.content, rich) { (Some(c), true) => (c), _ => (o.text) }
                }
            }
        };
        crate::labelled(
            label,
            name,
            html! {
                span class="nojs-select" {
                    @if let Some(action) = search {
                        span class="nojs-select-search" {
                            (Input::search_box(&q_name, "Filter options", q).id(&q_id).placeholder("Filter"))
                            (ui.button("Filter").formmethod("get").formaction(action).formnovalidate())
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
            },
        )
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-select { display: inline-grid; gap: 0.4rem; }
.nojs-select-search { display: flex; gap: 0.4rem; }
.nojs-select-search input { flex: 1; min-width: 0; }
.nojs-select select, .nojs-select select::picker(select) { appearance: base-select; }
.nojs-select select { min-width: 12rem; }
/* base-select draws its own ::picker-icon; drop the gradient chevron from layout.rs. */
@supports (appearance: base-select) { .nojs-select select { background-image: none; padding-right: 0.75rem; } }
.nojs-select select::picker(select) {
  border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); padding: 0.25rem;
  background: var(--nojs-popover); color: var(--nojs-fg); box-shadow: var(--nojs-shadow-lg);
  max-height: 20rem;
}
.nojs-select option { padding: 0.375rem 0.5rem; border-radius: var(--nojs-radius-sm); font-size: 0.875rem; }
.nojs-select option:hover, .nojs-select option:focus-visible { background: var(--nojs-accent); color: var(--nojs-on-accent); }
.nojs-select option::checkmark { order: 1; margin-left: auto; }
.nojs-select optgroup { font-size: 0.75rem; font-weight: 500; color: var(--nojs-muted); padding: 0.375rem 0.5rem 0; }
.nojs-select optgroup option { font-weight: 400; color: var(--nojs-fg); }
.nojs-select-icon { display: inline-block; width: 1.25em; text-align: center; }
.nojs-swatch { display: inline-block; width: 1em; height: 1em; border-radius: 50%; vertical-align: -0.15em; margin-right: 0.4em; border: 1px solid var(--nojs-line); }
"#;
