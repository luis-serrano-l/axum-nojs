//! # Toggle group
//!
//! A row of pressable options, one picked (alignment) or several (bold, italic), part of the
//! form around it. Each option is a real radio button or checkbox, drawn as a segmented
//! button, so the form posts the choice and the browser handles the keys.
//!
//! **Platform features:** `<fieldset>` of `<input type="radio">` (one) or
//! `type="checkbox"` (several) inside their labels; `:checked` and `:focus-visible` on the
//! input style the face next to it (every browser).
//!
//! **Accessibility:** a `<fieldset>` named by its visually hidden `<legend>`; arrow keys move
//! between radios and Space toggles a checkbox, as the platform does; an icon-only option has
//! its text as a hidden label. Checked by axe-core in headless Firefox on every demo route,
//! both capability variants, light and dark (no serious or critical violation).
//!
//! **What it does not do without script:** apply a choice the moment it is pressed; the
//! surrounding form sends it (a submit button, or the enhancement script's in-place post).
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from_request("/editor", "align=center", "");
//! let m = ui.toggle_group("align", "Alignment").option("left", "Left").option("center", "Center");
//! let html = m.render().into_string();
//! assert!(html.contains(r#"type="radio" name="align" value="center" checked"#));
//!
//! let m = ui.toggle_group("style", "Text style").multiple()
//!     .option("bold", "Bold").icon(Icon::Bold)
//!     .option("italic", "Italic").icon(Icon::Italic)
//!     .value("bold");
//! let html = m.render().into_string();
//! assert!(html.contains(r#"type="checkbox" name="style" value="bold" checked"#));
//! // The same in `lui!`:
//! let same = lui! { ToggleGroup("style", "Text style") multiple {
//!     option "bold" "Bold" icon=(Icon::Bold);
//!     option "italic" "Italic" icon=(Icon::Italic);
//!     value "bold";
//! } };
//! assert_eq!(same.into_string(), html);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::icon::Glyph;
use crate::props::{Prop, PropKind};

/// Pressable options, made by [`Ui::toggle_group`].
///
/// **Setters.** Values and items: `.option(..)`, `.icon(..)`, `.value(..)`; switches: `.multiple()`.
#[derive(Clone, Debug)]
pub struct ToggleGroup<'a> {
    name: &'a str,
    label: &'a str,
    items: Vec<(&'a str, &'a str, Option<Glyph<'a>>)>,
    picked: Vec<String>,
    multi: bool,
}

impl ToggleGroup<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("option", PropKind::Item, "value: &'a str, text: &'a str")
            .attr("value")
            .doc("An option posting `value`, shown as `text`."),
        Prop::new("icon", PropKind::Modifier, "icon: impl Into<Glyph<'a>>")
            .doc("The option added last shows this icon, its text becoming a hidden label."),
        Prop::new("value", PropKind::Value, "value: &'a str")
            .attr("checked")
            .doc("Press this option (again for more with `.multiple()`); the query's own values by default."),
        Prop::new("multiple", PropKind::Switch, "")
            .attr("type")
            .doc("Several options may be pressed (checkboxes, not radios)."),
    ];
}

impl Ui {
    /// Options posted as `name`, the group named `label` for assistive tech. The pressed ones
    /// start from the query's `name` values.
    pub fn toggle_group<'a>(&self, name: &'a str, label: &'a str) -> ToggleGroup<'a> {
        ToggleGroup {
            name,
            label,
            items: Vec::new(),
            picked: self.params(name).map(str::to_string).collect(),
            multi: false,
        }
    }
}

impl<'a> ToggleGroup<'a> {
    /// An option posting `value`, shown as `text`.
    pub fn option(mut self, value: &'a str, text: &'a str) -> Self {
        self.items.push((value, text, None));
        self
    }

    /// The old name of [`Self::option`], kept for one release.
    #[deprecated(note = "use .option()")]
    pub fn item(self, value: &'a str, text: &'a str) -> Self {
        self.option(value, text)
    }

    /// The option added last shows this icon, its text becoming a hidden label.
    pub fn icon(mut self, icon: impl Into<Glyph<'a>>) -> Self {
        if let Some(last) = self.items.last_mut() {
            last.2 = Some(icon.into());
        }
        self
    }

    /// Press this option (again for more with `.multiple()`); the query's own values by default.
    pub fn value(mut self, value: &'a str) -> Self {
        if !self.multi {
            self.picked.clear();
        }
        self.picked.push(value.to_string());
        self
    }

    /// Several options may be pressed (checkboxes, not radios).
    pub fn multiple(mut self) -> Self {
        self.multi = true;
        self
    }

    /// The old name of [`Self::multiple`], kept for one release.
    #[deprecated(note = "use .multiple()")]
    pub fn multi(self) -> Self {
        self.multiple()
    }
}

impl Render for ToggleGroup<'_> {
    fn render(&self) -> Markup {
        let kind = if self.multi { "checkbox" } else { "radio" };
        html! {
            fieldset class="lui-toggle-group" {
                legend class="lui-sr" { (self.label) }
                @for (value, text, icon) in &self.items {
                    label class="lui-toggle-group-item" {
                        input class="lui-toggle-group-input" type=(kind) name=(self.name) value=(value)
                            checked[self.picked.iter().any(|p| p == value)];
                        span class="lui-toggle-group-face" {
                            @if let Some(i) = icon { (i.hidden()) span class="lui-sr" { (text) } } @else { (text) }
                        }
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn ToggleGroup: outline
/// segments, the pressed one on the accent.
pub const CSS: &str = r#"
.lui-toggle-group { display: inline-flex; justify-self: start; align-self: start; width: fit-content; margin: 0; padding: 0; border: 1px solid var(--lui-input); border-radius: var(--lui-radius); overflow: hidden; box-shadow: var(--lui-shadow-xs); }
.lui-toggle-group-item { position: relative; display: inline-flex; }
.lui-toggle-group-item + .lui-toggle-group-item { border-left: 1px solid var(--lui-input); }
.lui-toggle-group-input { position: absolute; opacity: 0; width: 1px; height: 1px; margin: 0; }
.lui-toggle-group-face { display: inline-flex; align-items: center; justify-content: center; gap: 0.375rem; min-width: 2.25rem; height: 2.25rem; padding: 0 0.75rem; font-size: 0.875rem; font-weight: 500; color: var(--lui-fg); background: var(--lui-bg); cursor: pointer; }
.lui-toggle-group-face:hover { background: var(--lui-secondary); }
.lui-toggle-group-input:checked + .lui-toggle-group-face { background: var(--lui-accent); color: var(--lui-on-accent); }
.lui-toggle-group-input:focus-visible + .lui-toggle-group-face { outline: 2px solid var(--lui-ring); outline-offset: -2px; }
"#;
