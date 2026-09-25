//! # Pager
//!
//! A long list delivered a page at a time with a "Load more" link, no script.
//!
//! **Platform features:**
//! - Ordinary links to `?page=n`. The key stays bare: a load-more list has no id, and a page
//!   holds one. A paged table beside it pages by its own `page.<id>` (see [`crate::table`]),
//!   so the two no longer move together.
//! - `view-transition-name` on the list, together with the layout's
//!   `@view-transition { navigation: auto }` (Chrome 126+, Safari 18.2+) or the enhancement
//!   script's `startViewTransition`, so the new rows fade in under the old ones. The button
//!   has no name on purpose: a named button morphs from its old spot to its new one, which
//!   reads as a green block sliding down over the fresh rows.
//! - `scroll-margin` + fragment `#more` keeps the viewport on the new rows after navigation.
//!
//! **Accessibility:** a real link for "Load more"; the new rows land after an anchor so focus
//! and reading continue there. Checked by axe-core in headless Firefox on every demo route,
//! both capability variants, light and dark (no serious or critical violation).
//!
//! **What it does not do without script:** infinite scroll or keeping the scroll position
//! across pages.
//!
//! **Fallback:** without `Caps::ViewTransitions` the transition names are omitted and the page
//! navigates normally; the `#more` fragment still scrolls to the new rows.
//!
//! **Finding:** true infinite scroll (loading on scroll) is impossible without script.
//! Cumulative pages (`?page=3` shows rows 1..3×N) give the same *feel* with one click per page.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! // `?page=2` of 30 rows, 10 at a time: rows 0 to 19 are shown.
//! let ui = Ui::from_request("/list", "page=2", "");
//! let list = ui.pager("/list", 30).per_page(10);
//! assert_eq!(list.shown(), 20);
//! let m = list.rows(|i| html! { "Row " (i + 1) }).render().into_string();
//! assert!(m.contains("Row 20") && !m.contains("Row 21") && m.contains("?page=3#more"));
//! // The same in `lui!`:
//! let same = lui! { Pager("/list", 30) per_page=10 rows=|i| { "Row " (i + 1) }; };
//! assert_eq!(same.into_string(), m);
//! ```

use std::fmt;
use std::rc::Rc;

use maud::{Markup, Render, html};

use crate::button::Button;
use crate::i18n::{Strings, Text};
use crate::props::{Prop, PropKind};
use crate::{Cap, Caps, Ui, enhance};

/// A load-more list, made by [`Ui::pager`]: pages 1 to `?page=` of the rows, 10 per page
/// unless told otherwise.
///
/// **Setters.** Values and items: `.rows(..)`, `.per_page(..)`.
#[derive(Clone)]
pub struct Pager<'a> {
    caps: Caps,
    strings: &'static Strings,
    href: &'a str,
    total: usize,
    page: usize,
    per_page: usize,
    row: Option<Rc<dyn Fn(usize) -> Markup + 'a>>,
}

impl Pager<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("per_page", PropKind::Number, "per_page: usize")
            .default("10")
            .doc("Rows per page."),
        Prop::new(
            "rows",
            PropKind::Value,
            "row: impl Fn(usize) -> Markup + 'a",
        )
        .doc("Row `i` (0-based) of the list."),
    ];
}

/// The row closure is shown as `<fn>`: a builder is still printable data.
impl fmt::Debug for Pager<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pager")
            .field("caps", &self.caps)
            .field("href", &self.href)
            .field("total", &self.total)
            .field("page", &self.page)
            .field("per_page", &self.per_page)
            .field("row", &self.row.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl Ui {
    /// A list of `total` rows whose "Load more" link goes to `href?page=n`; the page shown
    /// is this request's `?page=`.
    pub fn pager<'a>(&self, href: &'a str, total: usize) -> Pager<'a> {
        let page = self
            .param("page")
            .and_then(|p| p.parse().ok())
            .unwrap_or(1)
            .max(1);
        Pager {
            caps: self.caps,
            strings: self.strings,
            href,
            total,
            page,
            per_page: 10,
            row: None,
        }
    }
}

impl<'a> Pager<'a> {
    /// Rows per page.
    pub fn per_page(mut self, per_page: usize) -> Self {
        self.per_page = per_page.max(1);
        self
    }

    /// How many rows the page shows: every row of pages 1 to `?page=`.
    pub fn shown(&self) -> usize {
        (self.page * self.per_page).min(self.total)
    }

    /// Row `i` (0-based) of the list; called for each shown row.
    pub fn rows(mut self, row: impl Fn(usize) -> Markup + 'a) -> Self {
        self.row = Some(Rc::new(row));
        self
    }
}

impl Render for Pager<'_> {
    fn render(&self) -> Markup {
        let Pager {
            strings: _,
            caps,
            href,
            total,
            page,
            per_page,
            ref row,
        } = *self;
        let vt = caps.has(Cap::ViewTransitions);
        let shown = self.shown();
        let first_new = (page - 1) * per_page;
        let next = format!("{href}?page={}#more", page + 1);
        html! {
            div id=(enhance::swap_id("lui-pager", href)) data-lui="swap" class="lui-pager" {
                ol class="lui-pager-list" style=[vt.then_some("view-transition-name: lui-pager-list")] {
                    @if let Some(row) = row {
                        @for i in 0..shown {
                            @if i == first_new && page > 1 {
                                li id="more" class="lui-pager-anchor" { (row(i)) }
                            } @else {
                                li { (row(i)) }
                            }
                        }
                    }
                }
                p class="lui-note" { (self.strings.fill(Text::ShowingOf, &[&shown, &total])) }
                @if page * per_page < total {
                    (Button::link(caps, self.strings.get(Text::LoadMore), &next).class("lui-pager-more"))
                }
            }
        }
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-pager-list { margin: 0; padding-left: 1.5rem; font-size: 0.875rem; }
.lui-pager-list li { padding: 0.5rem 0; border-bottom: 1px solid var(--lui-line); }
.lui-pager-anchor { scroll-margin-top: 4rem; }
/* "Load more" is the outline button, full width under the list. */
.lui-pager-more { display: flex; margin-top: 1rem; }
"#;
