//! The demo routes, one file per group of the index (`site::COMPONENTS`). Each has a
//! `routes()` that `crate::router` merges; `PAGES`, the pages that only show their component
//! and a note, served by [`pages`]; and `PREVIEWS`, the live component of each of its other
//! pages. `site::preview` finds both for the index.

pub(crate) mod blocks;
pub(crate) mod disclosure;
pub(crate) mod feedback;
pub(crate) mod flows;
pub(crate) mod input;
pub(crate) mod navigation;
pub(crate) mod overlays;
pub(crate) mod own;
pub(crate) mod primitives;
pub(crate) mod server_state;
pub(crate) mod table;
pub(crate) mod theme;
pub(crate) mod widgets;

use axum::{Router, routing::get};
use loco_ui::prelude::*;

/// A component page's href and the function that draws its component: the same function the
/// page calls around its `// code:` markers, so the index and the page cannot drift apart.
pub(crate) type Preview = (&'static str, fn(&Ui) -> Markup);

/// A page that is its component and a note under it: href, the function that draws the
/// component (as in [`Preview`]) and the note, where `` `x` `` is code and `[text](href)` a
/// link. Its title is its entry's in `site::COMPONENTS`.
pub(crate) type Simple = (&'static str, fn(&Ui) -> Markup, &'static str);

/// One GET route per simple page, each drawing its component and its note.
pub(crate) fn pages(list: &'static [Simple]) -> Router {
    list.iter()
        .fold(Router::new(), |router, &(href, draw, note)| {
            let title = crate::site::title(href);
            router.route(
                href,
                get(move |ui: Ui| async move {
                    let body = lui! { Stack gap=6 {
                        (draw(&ui))
                        @if !note.is_empty() { p class="lui-note" { (crate::site::note(note)) } }
                    } };
                    crate::site::page(&ui, title, body)
                }),
            )
        })
}
