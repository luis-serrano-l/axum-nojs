//! # Sidebar
//!
//! An app's navigation as a column: groups under small headings, each link with an optional
//! icon and a count, the link to the current path highlighted. Put it in a
//! [`Drawer`](crate::drawer) `.sidebar()` (or `ui.app_shell`) to have it collapse to a drawer
//! on narrow screens, or anywhere a column fits.
//!
//! **Platform features:** a `<nav>` of lists; `aria-current="page"` on the link to the current
//! path, compared on the server.
//!
//! **Accessibility:** a `<nav>` named by its label; each group is a list labelled by its
//! heading; the current page's link has `aria-current="page"`; icons are decorative, counts are
//! text. Checked by axe-core in headless Firefox on every demo route, both capability variants,
//! light and dark (no serious or critical violation).
//!
//! **What it does not do without script:** collapse to an icon rail with a toggle kept between
//! pages; wrap it in a drawer for the narrow-screen case.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from_request("/inbox", "", "");
//! let m = ui.sidebar("Mail").link("Inbox", "/inbox").link("Sent", "/sent").render().into_string();
//! assert!(m.contains(r#"<a href="/inbox" aria-current="page">"#));
//!
//! let m = ui.sidebar("Mail")
//!     .group("Mail")
//!     .link("Inbox", "/inbox").icon(Icon::Mail).badge("12")
//!     .link("Sent", "/sent")
//!     .group("Labels")
//!     .link("Work", "/labels/work");
//! let html = m.render().into_string();
//! assert!(html.contains("lui-sidebar-badge\">12<") && html.contains(">Labels</p>"));
//! // The same in `lui!`:
//! let same = lui! { Sidebar("Mail") {
//!     group "Mail";
//!     link "Inbox" "/inbox" icon=(Icon::Mail) badge="12";
//!     link "Sent" "/sent";
//!     group "Labels";
//!     link "Work" "/labels/work";
//! } };
//! assert_eq!(same.into_string(), html);
//! ```

use maud::{Markup, Render, html};

use crate::props::{Prop, PropKind};
use crate::{Icon, Ui, slug};

/// One link: its text, where it goes, an icon and a count.
#[derive(Clone, Debug)]
struct Link<'a> {
    text: &'a str,
    href: &'a str,
    icon: Option<Icon>,
    badge: Option<&'a str>,
}

/// A navigation column, made by [`Ui::sidebar`].
///
/// **Setters.** Values and items: `.group(..)`, `.link(..)`, `.icon(..)`, `.badge(..)`.
#[derive(Clone, Debug)]
pub struct Sidebar<'a> {
    label: &'a str,
    here: String,
    groups: Vec<(Option<&'a str>, Vec<Link<'a>>)>,
}

impl Sidebar<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("group", PropKind::Item, "heading: &'a str")
            .doc("Start a group of links under a small heading."),
        Prop::new("link", PropKind::Item, "text: &'a str, href: &'a str")
            .attr("href")
            .doc("A link; the one to the current path is marked current."),
        Prop::new("icon", PropKind::Modifier, "icon: Icon")
            .doc("An icon before the link added last."),
        Prop::new("badge", PropKind::Modifier, "text: &'a str")
            .doc("A count after the link added last."),
    ];
}

impl Ui {
    /// Navigation named `label` (for assistive tech).
    pub fn sidebar<'a>(&self, label: &'a str) -> Sidebar<'a> {
        Sidebar {
            label,
            here: self.state.path().to_string(),
            groups: vec![(None, Vec::new())],
        }
    }
}

impl<'a> Sidebar<'a> {
    /// Start a group of links under a small heading.
    pub fn group(mut self, heading: &'a str) -> Self {
        self.groups.push((Some(heading), Vec::new()));
        self
    }

    /// A link; the one to the current path is marked current.
    pub fn link(mut self, text: &'a str, href: &'a str) -> Self {
        if let Some((_, links)) = self.groups.last_mut() {
            links.push(Link {
                text,
                href,
                icon: None,
                badge: None,
            });
        }
        self
    }

    /// An icon before the link added last.
    pub fn icon(mut self, icon: Icon) -> Self {
        if let Some(l) = self.groups.last_mut().and_then(|g| g.1.last_mut()) {
            l.icon = Some(icon);
        }
        self
    }

    /// A count after the link added last.
    pub fn badge(mut self, text: &'a str) -> Self {
        if let Some(l) = self.groups.last_mut().and_then(|g| g.1.last_mut()) {
            l.badge = Some(text);
        }
        self
    }
}

impl Render for Sidebar<'_> {
    fn render(&self) -> Markup {
        let root = format!("lui-sidebar-{}", slug(self.label));
        html! {
            nav class="lui-sidebar" aria-label=(self.label) {
                @for (i, (heading, links)) in self.groups.iter().enumerate().filter(|(_, g)| g.0.is_some() || !g.1.is_empty()) {
                    @let heading_id = format!("{root}-{i}");
                    div class="lui-sidebar-group" {
                        @if let Some(h) = heading { p class="lui-sidebar-heading" id=(heading_id) { (h) } }
                        ul aria-labelledby=[heading.map(|_| heading_id.as_str())] {
                            @for l in links {
                                li {
                                    a href=(l.href) aria-current=[(l.href == self.here).then_some("page")] {
                                        @if let Some(icon) = l.icon { (icon) }
                                        span class="lui-sidebar-text" { (l.text) }
                                        @if let Some(b) = l.badge { span class="lui-sidebar-badge" { (b) } }
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

/// Styles for this component; included in [`crate::stylesheet`]. shadcn Sidebar: small muted
/// group labels, full-width rows, the current one on the accent.
pub const CSS: &str = r#"
.lui-sidebar { display: grid; gap: calc(var(--lui-space) * 2); font-size: 0.875rem; }
.lui-sidebar-group ul { list-style: none; margin: 0; padding: 0; display: grid; gap: 0.125rem; }
.lui-sidebar-heading { margin: 0 0 0.25rem; padding: 0 0.5rem; font-size: 0.75rem; font-weight: 500; color: var(--lui-muted); }
.lui-sidebar a { display: flex; align-items: center; gap: 0.5rem; padding: 0.375rem 0.5rem; border-radius: var(--lui-radius-sm); color: var(--lui-fg); text-decoration: none; }
.lui-sidebar a:hover { background: var(--lui-accent); color: var(--lui-on-accent); }
.lui-sidebar a[aria-current="page"] { background: var(--lui-accent); color: var(--lui-on-accent); font-weight: 500; }
.lui-sidebar-text { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.lui-sidebar-badge { font-size: 0.75rem; font-variant-numeric: tabular-nums; color: inherit; }
"#;
