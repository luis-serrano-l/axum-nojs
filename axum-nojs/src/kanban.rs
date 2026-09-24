//! # Kanban
//!
//! A board of columns and cards. Moving a card is a form post: each card carries arrow buttons
//! to the column on its left and on its right, posting `card=<key>&to=<column>`; the route
//! saves the move and redirects back. A column can have a work-in-progress limit.
//!
//! **Platform features:** `<form method="post">` with `name`/`value` buttons, Post/Redirect/Get;
//! each column is a `<section>` with a heading and an ordered list; the board scrolls sideways
//! (`overflow-x: auto`, `scroll-snap-type`, Chrome 69, Firefox 68, Safari 11) on a narrow
//! screen; `view-transition-name` per card (Chrome 111, Firefox 144, Safari 18) so, with the
//! enhancement script, a moved card slides to its new column.
//!
//! **What it does not do without script:** drag and drop, or reorder cards within a column
//! (the server decides the order: a moved card goes last in its new column).
//!
//! **Fallback:** without view transitions a card is simply in its new column after the move.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::from(Caps::all());
//! let board = ui.kanban("/board/move")
//!     .column("todo", "To do").card("c1", "Write the docs")
//!     .column("doing", "Doing").limit(2).card("c2", "Calendar").note("M23").card("c3", "Upload")
//!     .column("done", "Done");
//! let m = board.render().into_string();
//! assert!(m.contains(r#"name="card" value="c1""#) && m.contains(r#"name="to" value="doing""#));
//! assert!(m.contains("2 / 2") && !m.contains(r#"value="todo" aria-label="Move Write"#));
//! ```

use maud::{Markup, Render, html};

use crate::button::Button;
use crate::{Cap, Icon, Ui, enhance, slug};

/// A card: its key (what the move posts), title and small print.
#[derive(Clone, Debug)]
struct Card<'a> {
    key: &'a str,
    title: &'a str,
    note: Option<&'a str>,
}

/// A column: its key (the `to` a move posts), title, limit and cards.
#[derive(Clone, Debug)]
struct Column<'a> {
    key: &'a str,
    title: &'a str,
    limit: Option<usize>,
    cards: Vec<Card<'a>>,
}

/// A board, made by [`Ui::kanban`]. Add columns with [`Kanban::column`] and cards, into the
/// column added last, with [`Kanban::card`].
///
/// **Setters.** Values and items: `.column(..)`, `.limit(..)`, `.card(..)`, `.note(..)`.
#[derive(Clone, Debug)]
pub struct Kanban<'a> {
    ui: &'a Ui,
    action: &'a str,
    columns: Vec<Column<'a>>,
}

impl Ui {
    /// A board whose moves post to `action`.
    pub fn kanban<'a>(&'a self, action: &'a str) -> Kanban<'a> {
        Kanban {
            ui: self,
            action,
            columns: Vec::new(),
        }
    }
}

impl<'a> Kanban<'a> {
    /// A column: `key` is what a move to it posts as `to`, `title` its heading.
    pub fn column(mut self, key: &'a str, title: &'a str) -> Self {
        self.columns.push(Column {
            key,
            title,
            limit: None,
            cards: Vec::new(),
        });
        self
    }

    /// A work-in-progress limit for the column added last: its count reads `n / limit` and
    /// turns red past it (the server decides whether to refuse a move).
    pub fn limit(mut self, limit: usize) -> Self {
        if let Some(c) = self.columns.last_mut() {
            c.limit = Some(limit);
        }
        self
    }

    /// A card in the column added last: `key` is what a move posts as `card`.
    pub fn card(mut self, key: &'a str, title: &'a str) -> Self {
        if let Some(c) = self.columns.last_mut() {
            c.cards.push(Card {
                key,
                title,
                note: None,
            });
        }
        self
    }

    /// Small print under the card added last: an owner, a due date, a tag.
    pub fn note(mut self, text: &'a str) -> Self {
        if let Some(card) = self.columns.last_mut().and_then(|c| c.cards.last_mut()) {
            card.note = Some(text);
        }
        self
    }
}

impl Render for Kanban<'_> {
    fn render(&self) -> Markup {
        let caps = self.ui.caps;
        let vt = caps.has(Cap::ViewTransitions);
        let root = enhance::swap_id("nojs-kanban", self.action);
        let cols = &self.columns;
        html! {
            div id=(root) data-nojs="swap" class="nojs-kanban" {
                @for (i, col) in cols.iter().enumerate() {
                    @let heading = format!("{root}-{}", slug(col.key));
                    @let over = col.limit.is_some_and(|l| col.cards.len() > l);
                    section class="nojs-kanban-column" aria-labelledby=(heading) {
                        header class="nojs-kanban-head" {
                            h3 id=(heading) { (col.title) }
                            span class={ "nojs-kanban-count" @if over { " nojs-kanban-over" } } {
                                (col.cards.len()) @if let Some(l) = col.limit { " / " (l) }
                                @if over { span class="nojs-sr" { ", over the limit" } }
                            }
                        }
                        @if col.cards.is_empty() {
                            p class="nojs-kanban-empty" { "No cards" }
                        } @else {
                            ol class="nojs-kanban-cards" {
                                @for card in &col.cards {
                                    li class="nojs-kanban-card" style=[vt.then(|| format!("view-transition-name: nojs-kanban-{}", slug(card.key)))] {
                                        p class="nojs-kanban-title" { (card.title) }
                                        @if let Some(n) = card.note { p class="nojs-kanban-note" { (n) } }
                                        form method="post" action=(self.action) class="nojs-kanban-move" {
                                            input type="hidden" name="card" value=(card.key);
                                            @if let Some(prev) = i.checked_sub(1).and_then(|p| cols.get(p)) {
                                                @let label = format!("Move {} to {}", card.title, prev.title);
                                                (Button::new(caps, "").ghost().small().icon().name("to").value(prev.key).label(&label).content(html! { (Icon::ArrowLeft) }))
                                            }
                                            @if let Some(next) = cols.get(i + 1) {
                                                @let label = format!("Move {} to {}", card.title, next.title);
                                                (Button::new(caps, "").ghost().small().icon().name("to").value(next.key).label(&label).content(html! { (Icon::ArrowRight) }))
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. Columns on the surface, cards
/// as small shadcn cards; the board scrolls sideways when the columns do not fit.
pub const CSS: &str = r#"
.nojs-kanban {
  display: grid; grid-auto-flow: column; grid-auto-columns: minmax(15rem, 1fr); gap: calc(var(--nojs-space) * 2);
  overflow-x: auto; scroll-snap-type: x mandatory; padding-bottom: 0.5rem;
}
.nojs-kanban-column {
  display: grid; align-content: start; gap: 0.5rem; padding: 0.75rem; scroll-snap-align: start;
  background: var(--nojs-surface); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius-lg);
}
.nojs-kanban-head { display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; padding: 0 0.25rem; }
.nojs-kanban-head h3 { margin: 0; font-size: 0.875rem; font-weight: 600; }
.nojs-kanban-count { font-size: 0.75rem; font-weight: 500; color: var(--nojs-muted); font-variant-numeric: tabular-nums; }
.nojs-kanban-over { color: var(--nojs-danger); }
.nojs-kanban-cards { list-style: none; margin: 0; padding: 0; display: grid; gap: 0.5rem; }
.nojs-kanban-card {
  display: grid; grid-template-columns: 1fr auto; align-items: start; gap: 0.25rem 0.5rem; padding: 0.625rem 0.75rem;
  background: var(--nojs-card); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); box-shadow: var(--nojs-shadow-xs);
}
.nojs-kanban-title { grid-column: 1; margin: 0; font-size: 0.875rem; font-weight: 500; }
.nojs-kanban-note { grid-column: 1; margin: 0; font-size: 0.75rem; color: var(--nojs-muted); }
.nojs-kanban-move { grid-column: 2; grid-row: 1 / span 2; display: flex; margin: -0.25rem -0.375rem 0 0; }
.nojs-kanban-empty { margin: 0; padding: 1rem; text-align: center; font-size: 0.875rem; color: var(--nojs-muted); border: 1px dashed var(--nojs-line); border-radius: var(--nojs-radius); }
"#;
