//! # Icon
//!
//! A small set of inline SVG icons, drawn in `currentColor` so they take the colour of the
//! text around them: the ones buttons, menus, tables and alerts need. The shapes are
//! [Lucide](https://lucide.dev)'s (ISC licence), the set shadcn/ui uses.
//!
//! **Platform features:** inline `<svg>` (no icon font, no request, no sprite); an icon is
//! `aria-hidden` unless it is given a `.label()`, which makes it `role="img"` with that name.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! let check = ui.icon(Icon::Check).render().into_string();
//! assert!(check.starts_with(r#"<svg class="nojs-icon" aria-hidden="true""#));
//! // An icon that is the only content of a link or a button needs a name.
//! let warn = ui.icon(Icon::TriangleAlert).label("Warning").render().into_string();
//! assert!(warn.contains(r#"role="img" aria-label="Warning""#));
//! // An icon on its own renders decoratively, so it drops into any `html!`.
//! let m = html! { (Icon::Plus) " Add" };
//! assert!(m.into_string().contains("M12 5v14"));
//! ```

use maud::{Markup, PreEscaped, Render, html};

use crate::Ui;

/// Which icon to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Icon {
    ArrowLeft,
    ArrowRight,
    Calendar,
    Check,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    CircleCheck,
    Copy,
    Download,
    Ellipsis,
    ExternalLink,
    File,
    House,
    Info,
    Mail,
    Menu,
    Minus,
    Moon,
    Pencil,
    Plus,
    Search,
    Sun,
    Trash,
    TriangleAlert,
    Upload,
    User,
    X,
}

impl Icon {
    /// Every icon, in the order the enum lists them.
    pub const ALL: [Icon; 29] = [
        Icon::ArrowLeft,
        Icon::ArrowRight,
        Icon::Calendar,
        Icon::Check,
        Icon::ChevronDown,
        Icon::ChevronLeft,
        Icon::ChevronRight,
        Icon::ChevronUp,
        Icon::CircleCheck,
        Icon::Copy,
        Icon::Download,
        Icon::Ellipsis,
        Icon::ExternalLink,
        Icon::File,
        Icon::House,
        Icon::Info,
        Icon::Mail,
        Icon::Menu,
        Icon::Minus,
        Icon::Moon,
        Icon::Pencil,
        Icon::Plus,
        Icon::Search,
        Icon::Sun,
        Icon::Trash,
        Icon::TriangleAlert,
        Icon::Upload,
        Icon::User,
        Icon::X,
    ];

    /// The icon's Lucide name (`"chevron-down"`).
    pub fn name(self) -> &'static str {
        self.parts().0
    }

    /// The name and the SVG elements inside a 24×24 view box.
    fn parts(self) -> (&'static str, &'static str) {
        match self {
            Icon::ArrowLeft => (
                "arrow-left",
                r#"<path d="m12 19-7-7 7-7"/><path d="M19 12H5"/>"#,
            ),
            Icon::ArrowRight => (
                "arrow-right",
                r#"<path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>"#,
            ),
            Icon::Calendar => (
                "calendar",
                r#"<path d="M8 2v4"/><path d="M16 2v4"/><rect width="18" height="18" x="3" y="4" rx="2"/><path d="M3 10h18"/>"#,
            ),
            Icon::Check => ("check", r#"<path d="M20 6 9 17l-5-5"/>"#),
            Icon::ChevronDown => ("chevron-down", r#"<path d="m6 9 6 6 6-6"/>"#),
            Icon::ChevronLeft => ("chevron-left", r#"<path d="m15 18-6-6 6-6"/>"#),
            Icon::ChevronRight => ("chevron-right", r#"<path d="m9 18 6-6-6-6"/>"#),
            Icon::ChevronUp => ("chevron-up", r#"<path d="m18 15-6-6-6 6"/>"#),
            Icon::CircleCheck => (
                "circle-check",
                r#"<circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/>"#,
            ),
            Icon::Copy => (
                "copy",
                r#"<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>"#,
            ),
            Icon::Download => (
                "download",
                r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="m7 10 5 5 5-5"/><path d="M12 15V3"/>"#,
            ),
            Icon::Ellipsis => (
                "ellipsis",
                r#"<circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/>"#,
            ),
            Icon::ExternalLink => (
                "external-link",
                r#"<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>"#,
            ),
            Icon::File => (
                "file",
                r#"<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z"/><path d="M14 2v4a2 2 0 0 0 2 2h4"/>"#,
            ),
            Icon::House => (
                "house",
                r#"<path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>"#,
            ),
            Icon::Info => (
                "info",
                r#"<circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>"#,
            ),
            Icon::Mail => (
                "mail",
                r#"<rect width="20" height="16" x="2" y="4" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/>"#,
            ),
            Icon::Menu => (
                "menu",
                r#"<path d="M4 6h16"/><path d="M4 12h16"/><path d="M4 18h16"/>"#,
            ),
            Icon::Minus => ("minus", r#"<path d="M5 12h14"/>"#),
            Icon::Moon => ("moon", r#"<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>"#),
            Icon::Pencil => (
                "pencil",
                r#"<path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/>"#,
            ),
            Icon::Plus => ("plus", r#"<path d="M5 12h14"/><path d="M12 5v14"/>"#),
            Icon::Search => (
                "search",
                r#"<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>"#,
            ),
            Icon::Sun => (
                "sun",
                r#"<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>"#,
            ),
            Icon::Trash => (
                "trash",
                r#"<path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>"#,
            ),
            Icon::TriangleAlert => (
                "triangle-alert",
                r#"<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/>"#,
            ),
            Icon::Upload => (
                "upload",
                r#"<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="m17 8-5-5-5 5"/><path d="M12 3v12"/>"#,
            ),
            Icon::User => (
                "user",
                r#"<path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>"#,
            ),
            Icon::X => ("x", r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#),
        }
    }
}

/// An icon with an optional accessible name, made by [`Ui::icon`].
///
/// **Setters.** Values and items: `.label(..)`.
#[derive(Clone, Copy, Debug)]
pub struct IconMark<'a> {
    icon: Icon,
    label: Option<&'a str>,
}

impl Ui {
    /// The `icon`, decorative (`aria-hidden`) unless given a `.label()`.
    pub fn icon<'a>(&self, icon: Icon) -> IconMark<'a> {
        IconMark { icon, label: None }
    }
}

impl<'a> IconMark<'a> {
    /// The name a screen reader says, for an icon that carries meaning on its own.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl Render for IconMark<'_> {
    fn render(&self) -> Markup {
        html! {
            svg class="nojs-icon" aria-hidden=[self.label.is_none().then_some("true")]
                role=[self.label.map(|_| "img")] aria-label=[self.label]
                xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                stroke-width="2" stroke-linecap="round" stroke-linejoin="round" data-icon=(self.icon.name()) {
                (PreEscaped(self.icon.parts().1))
            }
        }
    }
}

impl Render for Icon {
    fn render(&self) -> Markup {
        IconMark {
            icon: *self,
            label: None,
        }
        .render()
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn's size-4.
pub const CSS: &str = r#"
.nojs-icon { width: 1rem; height: 1rem; flex: none; vertical-align: -0.125em; pointer-events: none; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_has_a_distinct_name_and_shape() {
        let mut names: Vec<_> = Icon::ALL.iter().map(|i| i.name()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), Icon::ALL.len());
        for i in Icon::ALL {
            let svg = i.render().into_string();
            assert!(
                svg.ends_with("</svg>") && (svg.contains("<path") || svg.contains("<circle")),
                "{svg}"
            );
        }
    }
}
