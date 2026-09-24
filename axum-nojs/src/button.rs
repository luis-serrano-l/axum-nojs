//! # Button
//!
//! The one button every other component builds on: shadcn's outline button by default, with
//! primary, danger and ghost tones, a small size and a square icon size. `ui.link_button`
//! gives a link the same look, for actions that are a navigation (GET) rather than a post.
//!
//! **Platform features:** `<button>` with its `type`, `name`/`value` (sent with the form that
//! submits it), `form=` (submit a form elsewhere on the page), invoker commands
//! (`command`/`commandfor`, Chrome 135, Firefox 144, Safari 26.2) and `popovertarget`
//! (Chrome 114, Firefox 125, Safari 17). A loading button is `disabled` and `aria-busy`, with a
//! CSS-only spinner that slows down under `prefers-reduced-motion`.
//!
//! **What it does not do without script:** turn itself into a loading button while its form
//! posts; the server sets `.loading(true)` on the page it renders (the enhancement script
//! marks a posting form `aria-busy` on its own).
//!
//! **Fallback:** a popover command (`toggle-popover`, `show-popover`, `hide-popover`) in a
//! browser without invoker commands is written as `popovertarget` and `popovertargetaction`,
//! which popover browsers have had since 2023. Other commands (`show-modal`, `close`) have
//! no attribute fallback; the dialog component uses a `:target` link there instead.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::from(Caps::all());
//! let save = ui.button("Save").primary().render().into_string();
//! assert!(save.contains(r#"class="nojs-button nojs-button-primary""#) && save.contains(r#"type="submit""#));
//! // A small ghost icon button that toggles a popover, and a link that looks like a button.
//! let more = ui.button("\u{22ef}").ghost().small().icon().label("More").command("toggle-popover", "menu");
//! let more = more.render().into_string();
//! assert!(more.contains(r#"commandfor="menu""#) && more.contains(r#"aria-label="More""#));
//! let docs = ui.link_button("Read the docs", "/docs").render().into_string();
//! assert!(docs.starts_with("<a") && docs.contains(r#"href="/docs""#));
//! // The server knows the job is still running, so the page it renders says so.
//! let busy = ui.button("Export").loading(true).render().into_string();
//! assert!(busy.contains("disabled") && busy.contains(r#"aria-busy="true""#));
//! ```

use maud::{Markup, Render, html};

use crate::{Cap, Caps, Ui};

/// The colour a button takes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Tone {
    /// shadcn "outline": the page background, a border, the accent surface on hover.
    #[default]
    Outline,
    Primary,
    Danger,
    Ghost,
}

/// A `<button>` or a link that looks like one, made by [`Ui::button`] or [`Ui::link_button`].
#[derive(Clone, Debug)]
pub struct Button<'a> {
    caps: Caps,
    text: &'a str,
    content: Option<Markup>,
    href: Option<&'a str>,
    tone: Tone,
    small: bool,
    icon: bool,
    kind: Option<&'static str>,
    command: Option<(&'a str, &'a str)>,
    popovertarget: Option<&'a str>,
    form: Option<&'a str>,
    name: Option<&'a str>,
    value: Option<&'a str>,
    label: Option<&'a str>,
    class: Option<&'a str>,
    disabled: bool,
    loading: bool,
    attrs: Attrs<'a>,
}

/// The less common attributes, each set by the setter of the same name.
#[derive(Clone, Debug, Default)]
struct Attrs<'a> {
    id: Option<&'a str>,
    role: Option<&'a str>,
    title: Option<&'a str>,
    style: Option<String>,
    aria_haspopup: Option<&'a str>,
    aria_pressed: Option<bool>,
    accesskey: Option<&'a str>,
    aria_keyshortcuts: Option<&'a str>,
    formmethod: Option<&'a str>,
    formaction: Option<&'a str>,
    formnovalidate: bool,
    rel: Option<&'a str>,
    current: bool,
}

impl Ui {
    /// A button reading `text`. It submits its form unless it is given a command or a
    /// popover target, which make it a plain `type="button"`.
    pub fn button<'a>(&self, text: &'a str) -> Button<'a> {
        Button::new(self.caps, text)
    }

    /// A link to `href` that looks like a button. The button-only setters (`.submit()`,
    /// `.command()`, `.form()`, `.name()`, `.value()`, `.popovertarget()`) do nothing on it.
    pub fn link_button<'a>(&self, text: &'a str, href: &'a str) -> Button<'a> {
        Button {
            href: Some(href),
            ..self.button(text)
        }
    }
}

impl<'a> Button<'a> {
    /// A button for a component that holds only `caps` (they decide the popover fallback).
    pub(crate) fn new(caps: Caps, text: &'a str) -> Self {
        Button {
            caps,
            text,
            content: None,
            href: None,
            tone: Tone::Outline,
            small: false,
            icon: false,
            kind: None,
            command: None,
            popovertarget: None,
            form: None,
            name: None,
            value: None,
            label: None,
            class: None,
            disabled: false,
            loading: false,
            attrs: Attrs::default(),
        }
    }

    /// A link for a component that holds only `caps`.
    pub(crate) fn link(caps: Caps, text: &'a str, href: &'a str) -> Self {
        Button {
            href: Some(href),
            ..Button::new(caps, text)
        }
    }

    /// The main action of a form or page: filled with `--nojs-primary`.
    pub fn primary(mut self) -> Self {
        self.tone = Tone::Primary;
        self
    }

    /// Destroys or removes something: filled with `--nojs-danger`.
    pub fn danger(mut self) -> Self {
        self.tone = Tone::Danger;
        self
    }

    /// No border or fill until hovered: toolbars, row menus, close buttons.
    pub fn ghost(mut self) -> Self {
        self.tone = Tone::Ghost;
        self
    }

    /// 2rem tall instead of 2.25rem.
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    /// Square, for a glyph or an icon; give it a `.label()` for screen readers.
    pub fn icon(mut self) -> Self {
        self.icon = true;
        self
    }

    /// `type="submit"`, the default unless a command or popover target is set.
    pub fn submit(mut self) -> Self {
        self.kind = Some("submit");
        self
    }

    /// `type="reset"`: puts the form's fields back to the values the page was served with.
    pub fn reset(mut self) -> Self {
        self.kind = Some("reset");
        self
    }

    /// An invoker command (`command`, `commandfor`): `"show-modal"`, `"close"`,
    /// `"toggle-popover"` and the like on the element with id `target`.
    pub fn command(mut self, command: &'a str, target: &'a str) -> Self {
        self.command = Some((command, target));
        self
    }

    /// Toggle the popover with this id (`popovertarget`).
    pub fn popovertarget(mut self, id: &'a str) -> Self {
        self.popovertarget = Some(id);
        self
    }

    /// Submit the form with this id, wherever the button sits in the page (`form=`).
    pub fn form(mut self, id: &'a str) -> Self {
        self.form = Some(id);
        self
    }

    /// Sent as `name=value` with the form when this button submits it.
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// See [`Button::name`].
    pub fn value(mut self, value: &'a str) -> Self {
        self.value = Some(value);
        self
    }

    /// `aria-label`: what an icon button does, for screen readers.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// One more class after the button's own, for a component's part name.
    pub fn class(mut self, class: &'a str) -> Self {
        self.class = Some(class);
        self
    }

    /// Greyed out and not clickable; a disabled link loses its `href`.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// A spinner before the text, `disabled` and `aria-busy`, while `on`.
    pub fn loading(mut self, on: bool) -> Self {
        self.loading = on;
        self
    }

    /// Markup to show instead of the text: an icon beside it, a count, a formatted string.
    pub fn content(mut self, markup: Markup) -> Self {
        self.content = Some(markup);
        self
    }

    /// The element's `id`.
    pub fn id(mut self, id: &'a str) -> Self {
        self.attrs.id = Some(id);
        self
    }

    /// `role`, for a button that is a menu item or a tab, or a link that acts as a button.
    pub fn role(mut self, role: &'a str) -> Self {
        self.attrs.role = Some(role);
        self
    }

    /// `title`: the tooltip.
    pub fn title(mut self, title: &'a str) -> Self {
        self.attrs.title = Some(title);
        self
    }

    /// Inline `style`, for a per-element custom property or anchor name.
    pub fn style(mut self, style: impl Into<String>) -> Self {
        self.attrs.style = Some(style.into());
        self
    }

    /// `aria-haspopup` (`"menu"`, `"dialog"`).
    pub fn aria_haspopup(mut self, kind: &'a str) -> Self {
        self.attrs.aria_haspopup = Some(kind);
        self
    }

    /// `aria-pressed`, for a toggle button; set from the state it shows.
    pub fn pressed(mut self, on: bool) -> Self {
        self.attrs.aria_pressed = Some(on);
        self
    }

    /// `accesskey`.
    pub fn accesskey(mut self, key: &'a str) -> Self {
        self.attrs.accesskey = Some(key);
        self
    }

    /// `aria-keyshortcuts`, the shortcut spelled out for assistive technology.
    pub fn aria_keyshortcuts(mut self, keys: &'a str) -> Self {
        self.attrs.aria_keyshortcuts = Some(keys);
        self
    }

    /// `formmethod`: submit the form with this method instead of its own.
    pub fn formmethod(mut self, method: &'a str) -> Self {
        self.attrs.formmethod = Some(method);
        self
    }

    /// `formaction`: submit the form to this URL instead of its own.
    pub fn formaction(mut self, action: &'a str) -> Self {
        self.attrs.formaction = Some(action);
        self
    }

    /// `formnovalidate`: submit without the browser's constraint checks.
    pub fn formnovalidate(mut self) -> Self {
        self.attrs.formnovalidate = true;
        self
    }

    /// `rel` of a link (`"prev"`, `"next"`).
    pub fn rel(mut self, rel: &'a str) -> Self {
        self.attrs.rel = Some(rel);
        self
    }

    /// `aria-current="page"`: the link to the page being shown; set from a condition.
    pub fn current(mut self, on: bool) -> Self {
        self.attrs.current = on;
        self
    }

    fn classes(&self) -> String {
        let mut c = String::from("nojs-button");
        match self.tone {
            Tone::Outline => {}
            Tone::Primary => c.push_str(" nojs-button-primary"),
            Tone::Danger => c.push_str(" nojs-button-danger"),
            Tone::Ghost => c.push_str(" nojs-button-ghost"),
        }
        if self.small {
            c.push_str(" nojs-button-small");
        }
        if self.icon {
            c.push_str(" nojs-button-icon");
        }
        if let Some(extra) = self.class {
            c.push(' ');
            c.push_str(extra);
        }
        c
    }
}

/// `toggle-popover` → `toggle`, the `popovertargetaction` with the same effect.
fn popover_action(command: &str) -> Option<&'static str> {
    match command {
        "toggle-popover" => Some("toggle"),
        "show-popover" => Some("show"),
        "hide-popover" => Some("hide"),
        _ => None,
    }
}

impl Render for Button<'_> {
    fn render(&self) -> Markup {
        let class = self.classes();
        let a = &self.attrs;
        let text = html! { @if let Some(m) = &self.content { (m) } @else { (self.text) } };
        let spinner =
            html! { @if self.loading { span class="nojs-button-spinner" aria-hidden="true" {} } };
        if let Some(href) = self.href {
            let off = self.disabled || self.loading;
            return html! {
                a class=(class) href=[(!off).then_some(href)] role=[a.role.or(off.then_some("link"))]
                    aria-disabled=[off.then_some("true")] aria-busy=[self.loading.then_some("true")]
                    aria-label=[self.label] id=[a.id] title=[a.title] style=[a.style.as_deref()]
                    rel=[a.rel] aria-current=[a.current.then_some("page")]
                    accesskey=[a.accesskey] aria-keyshortcuts=[a.aria_keyshortcuts] { (spinner) (text) }
            };
        }
        // Without invoker commands, a popover command becomes the older popovertarget pair.
        let (command, fallback) = match self.command {
            Some((cmd, target)) if !self.caps.has(Cap::Invokers) => match popover_action(cmd) {
                Some(action) => (None, Some((target, action))),
                None => (self.command, None),
            },
            other => (other, None),
        };
        let target = self.popovertarget.or(fallback.map(|(t, _)| t));
        let kind = self
            .kind
            .unwrap_or(if command.is_some() || target.is_some() {
                "button"
            } else {
                "submit"
            });
        html! {
            button type=(kind) class=(class)
                command=[command.map(|c| c.0)] commandfor=[command.map(|c| c.1)]
                popovertarget=[target] popovertargetaction=[fallback.map(|(_, a)| a)]
                form=[self.form] name=[self.name] value=[self.value] aria-label=[self.label]
                aria-busy=[self.loading.then_some("true")] disabled[self.disabled || self.loading]
                id=[a.id] role=[a.role] title=[a.title] style=[a.style.as_deref()]
                aria-haspopup=[a.aria_haspopup] aria-pressed=[a.aria_pressed.map(|p| if p { "true" } else { "false" })]
                accesskey=[a.accesskey] aria-keyshortcuts=[a.aria_keyshortcuts]
                formmethod=[a.formmethod] formaction=[a.formaction] formnovalidate[a.formnovalidate]
                { (spinner) (text) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. Bare `button` keeps the
/// outline look (and `button.nojs-primary`/`.nojs-danger` their fills) so a hand-written
/// button in a page matches; components build theirs with [`Ui::button`].
pub const CSS: &str = r#"
button, .nojs-button {
  display: inline-flex; align-items: center; justify-content: center; gap: 0.5rem;
  min-height: 2.25rem; padding: 0.375rem 1rem; white-space: nowrap; cursor: pointer;
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
  background: var(--nojs-bg); border: 1px solid var(--nojs-input); border-radius: var(--nojs-radius-sm);
  box-shadow: var(--nojs-shadow-xs); transition: background-color 0.15s, color 0.15s, box-shadow 0.15s;
}
.nojs-button { color: var(--nojs-fg); text-decoration: none; box-sizing: border-box; }
button:hover, .nojs-button:hover { background: var(--nojs-accent); color: var(--nojs-on-accent); }
button.nojs-primary, .nojs-button.nojs-button-primary { background: var(--nojs-primary); color: var(--nojs-on-primary); border-color: transparent; }
button.nojs-primary:hover, .nojs-button.nojs-button-primary:hover { background: color-mix(in srgb, var(--nojs-primary) 90%, transparent); color: var(--nojs-on-primary); }
button.nojs-danger, .nojs-button.nojs-button-danger { background: var(--nojs-danger); color: var(--nojs-on-primary); border-color: transparent; }
button.nojs-danger:hover, .nojs-button.nojs-button-danger:hover { background: color-mix(in srgb, var(--nojs-danger) 90%, transparent); color: var(--nojs-on-primary); }
.nojs-button.nojs-button-ghost { background: transparent; border-color: transparent; box-shadow: none; }
.nojs-button.nojs-button-ghost:hover { background: var(--nojs-accent); }
.nojs-button.nojs-button-small { min-height: 2rem; padding: 0.25rem 0.75rem; gap: 0.375rem; }
.nojs-button.nojs-button-icon { width: 2.25rem; min-width: 2.25rem; padding: 0; }
.nojs-button.nojs-button-icon.nojs-button-small { width: 2rem; min-width: 2rem; }
.nojs-button:focus-visible { border-color: var(--nojs-ring); }
.nojs-button[aria-disabled=true] { opacity: 0.5; cursor: not-allowed; pointer-events: none; }
.nojs-button[aria-busy=true] { cursor: progress; }
.nojs-button-spinner {
  width: 1rem; height: 1rem; flex: none; border-radius: 50%;
  border: 2px solid currentColor; border-right-color: transparent;
  animation: nojs-spin 0.6s linear infinite;
}
@keyframes nojs-spin { to { transform: rotate(1turn); } }
@media (prefers-reduced-motion: reduce) { .nojs-button-spinner { animation-duration: 1.5s; } }
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Caps;

    #[test]
    fn a_popover_command_falls_back_to_popovertarget() {
        let modern = Ui::from(Caps::all());
        let m = modern
            .button("Menu")
            .command("toggle-popover", "m")
            .render()
            .into_string();
        assert!(m.contains(r#"command="toggle-popover""#) && !m.contains("popovertarget"));
        let old = Ui::from(Caps::default().with(Cap::Popover));
        let o = old
            .button("Menu")
            .command("toggle-popover", "m")
            .render()
            .into_string();
        assert!(
            o.contains(r#"popovertarget="m""#) && o.contains(r#"popovertargetaction="toggle""#)
        );
        assert!(!o.contains("command=") && o.contains(r#"type="button""#));
        // No attribute says "show-modal" without invokers; the command is written as asked.
        let d = old
            .button("Open")
            .command("show-modal", "d")
            .render()
            .into_string();
        assert!(d.contains(r#"command="show-modal""#));
    }

    #[test]
    fn setters_become_attributes() {
        let ui = Ui::default();
        let b = ui
            .button("Delete")
            .danger()
            .form("f")
            .name("op")
            .value("rm")
            .disabled();
        let b = b.render().into_string();
        for part in [
            "nojs-button-danger",
            r#"form="f""#,
            r#"name="op""#,
            r#"value="rm""#,
            "disabled",
            r#"type="submit""#,
        ] {
            assert!(b.contains(part), "{part} missing in {b}");
        }
        let r = ui
            .button("Undo")
            .reset()
            .class("nojs-x")
            .render()
            .into_string();
        assert!(r.contains(r#"type="reset""#) && r.contains("nojs-button nojs-x"));
        let off = ui
            .link_button("Next", "/p/2")
            .disabled()
            .render()
            .into_string();
        assert!(!off.contains("href") && off.contains(r#"aria-disabled="true""#));
    }
}
