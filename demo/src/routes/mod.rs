//! The demo routes, one file per group of the index (`site::COMPONENTS`). Each has a
//! `routes()` that `crate::router` merges, and `PREVIEWS`, the live component of each of its
//! pages, which `site::preview` finds for the index.

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

use loco_ui::prelude::*;

/// A component page's href and the function that draws its component: the same function the
/// page calls around its `// code:` markers, so the index and the page cannot drift apart.
pub(crate) type Preview = (&'static str, fn(&Ui) -> Markup);
