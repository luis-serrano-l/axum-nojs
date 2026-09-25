//! # Avatar
//!
//! A round picture of a person, with their initials underneath for when there is no picture
//! or it fails to load.
//!
//! **Platform features:** the initials are painted first and the `<img>` is stacked over
//! them; an image that fails to load has `alt=""`, so it renders nothing and the initials
//! show through, with no `onerror` handler. `loading="lazy"` defers images below the fold.
//! The root is `role="img"` named after the person, so a screen reader says the name once.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** the initials are the fallback.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! let m = ui.avatar("Ada Lovelace").render().into_string();
//! assert!(m.contains(r#"role="img" aria-label="Ada Lovelace""#) && m.contains(">AL</span>"));
//! // The same in `nojs!`:
//! let same = nojs! { Avatar("Ada Lovelace"); };
//! assert_eq!(same.into_string(), m);
//! let m = ui.avatar("Grace Hopper").src("/img/grace.jpg").large().render().into_string();
//! assert!(m.contains(r#"<img src="/img/grace.jpg" alt="" loading="lazy">"#) && m.contains("nojs-avatar-large"));
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};

/// An avatar, made by [`Ui::avatar`].
///
/// **Setters.** Values and items: `.src(..)`; switches: `.small()`, `.large()`.
#[derive(Clone, Debug)]
pub struct Avatar<'a> {
    name: &'a str,
    src: Option<&'a str>,
    size: Option<&'static str>,
}

impl Avatar<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("src", PropKind::Value, "src: &'a str")
            .attr("src")
            .doc("The picture's URL."),
        Prop::new("small", PropKind::Switch, "")
            .doc("1.5rem across instead of 2rem, for lists and table rows."),
        Prop::new("large", PropKind::Switch, "").doc("3rem across, for a profile header."),
    ];
}

impl Ui {
    /// An avatar for the person called `name`: their initials, until `.src()` gives a
    /// picture.
    pub fn avatar<'a>(&self, name: &'a str) -> Avatar<'a> {
        Avatar {
            name,
            src: None,
            size: None,
        }
    }
}

impl<'a> Avatar<'a> {
    /// The picture's URL.
    pub fn src(mut self, src: &'a str) -> Self {
        self.src = Some(src);
        self
    }

    /// 1.5rem across instead of 2rem, for lists and table rows.
    pub fn small(mut self) -> Self {
        self.size = Some("nojs-avatar-small");
        self
    }

    /// 3rem across, for a profile header.
    pub fn large(mut self) -> Self {
        self.size = Some("nojs-avatar-large");
        self
    }
}

/// The first letter of the first and last words, upper-cased: `"Ada King Lovelace"` → `AL`.
fn initials(name: &str) -> String {
    let mut words = name.split_whitespace().filter_map(|w| w.chars().next());
    let first = words.next();
    let last = words.next_back();
    first
        .into_iter()
        .chain(last)
        .flat_map(char::to_uppercase)
        .collect()
}

impl Render for Avatar<'_> {
    fn render(&self) -> Markup {
        html! {
            span class={ "nojs-avatar" @if let Some(s) = self.size { " " (s) } } role="img" aria-label=(self.name) {
                span class="nojs-avatar-initials" aria-hidden="true" { (initials(self.name)) }
                @if let Some(src) = self.src { img src=(src) alt="" loading="lazy"; }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn: size-8, rounded-full,
/// the fallback on `--nojs-secondary`.
pub const CSS: &str = r#"
.nojs-avatar {
  position: relative; display: inline-grid; place-items: center; flex: none; overflow: hidden;
  width: 2rem; height: 2rem; border-radius: 9999px; background: var(--nojs-secondary); color: var(--nojs-fg);
  font-size: 0.75rem; font-weight: 500; line-height: 1; vertical-align: middle; user-select: none;
}
/* 2px past the edge on every side: the frame Firefox draws round a broken image falls
   outside the circle and is clipped away. */
.nojs-avatar img { position: absolute; inset: -2px; width: calc(100% + 4px); height: calc(100% + 4px); max-width: none; object-fit: cover; color: transparent; }
.nojs-avatar-small { width: 1.5rem; height: 1.5rem; font-size: 0.625rem; }
.nojs-avatar-large { width: 3rem; height: 3rem; font-size: 1rem; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_take_the_first_and_last_words() {
        assert_eq!(initials("Ada King Lovelace"), "AL");
        assert_eq!(initials("prince"), "P");
        assert_eq!(initials("  "), "");
        assert_eq!(initials("éric satie"), "ÉS");
    }
}
