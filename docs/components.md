# Write your own component

The components in `axum-nojs` are not special. Each is a builder that holds `&Ui`, renders
with `impl Render`, and ships a `CSS` const; a component of yours is the same, built from
the same primitives (`ui.button`, `ui.input`, `ui.card`, `ui.badge`, `Icon`, `ui.stack`,
`ui.cluster`, `ui.grid`, `ui.split`), so it looks and behaves like the rest and still needs no
script. This page builds one in about 30 lines and then lists the rules the library itself
follows.

## A newsletter box in 30 lines

A card with an email field and a button. The post answers with a redirect, and the box shows
its thank-you state from the URL; with the enhancement script it updates in place.

```rust
use axum_nojs::{enhance, prelude::*, slug};

/// A sign-up card: an email field and a button, or a thank-you once `?subscribed` is set.
pub struct Subscribe<'a> { ui: &'a Ui, action: &'a str, title: &'a str }

/// The extension trait, so a route writes `ui.subscribe(..)` like any built-in component.
pub trait SubscribeExt { fn subscribe<'a>(&'a self, action: &'a str) -> Subscribe<'a>; }
impl SubscribeExt for Ui {
    fn subscribe<'a>(&'a self, action: &'a str) -> Subscribe<'a> {
        Subscribe { ui: self, action, title: "Get the newsletter" }
    }
}

impl<'a> Subscribe<'a> {
    /// The card's heading.
    pub fn title(mut self, title: &'a str) -> Self { self.title = title; self }
}

impl Render for Subscribe<'_> {
    fn render(&self) -> Markup {
        let ui = self.ui;
        let body = if ui.param("subscribed").is_some() {
            html! { p { "Thanks, you are on the list." } }
        } else {
            html! { form method="post" action=(self.action) class="acme-subscribe-form" {
                (ui.input("email", "Email").email().required().hide_label()
                    .placeholder("you@example.com").class("acme-subscribe-email"))
                (ui.button("Subscribe").primary())
            } }
        };
        // A swap root: the enhancement script replaces just this box after the post.
        let id = enhance::swap_id("acme-subscribe", &slug(self.title));
        html! { div id=(id) data-nojs="swap" class="acme-subscribe" { (ui.card().title(self.title).body(body)) } }
    }
}

/// Styles: classes of your own, sizes and colours from the `--nojs-*` tokens.
pub const SUBSCRIBE_CSS: &str = "
.acme-subscribe-form { display: flex; gap: var(--nojs-space-2); }
.acme-subscribe-email { flex: 1; }
";

// The routes: the page ships the CSS once; the post redirects (Post/Redirect/Get).
fn home(ui: &Ui) -> Page {
    ui.page("Home", html! { (ui.subscribe("/subscribe")) }).css(SUBSCRIBE_CSS)
}
fn subscribe(ui: &Ui) -> Redirect {
    ui.redirect("/?subscribed=1").ok("You are subscribed.")
}

// What it renders, with and without the query.
let page = home(&Ui::from_request("/", "", "")).into_string();
assert!(page.contains(r#"id="acme-subscribe-get-the-newsletter" data-nojs="swap""#));
assert!(page.contains(r#"type="email""#) && page.contains(".acme-subscribe-email{flex:1}"));
let done = home(&Ui::from_request("/", "subscribed=1", "")).into_string();
assert!(done.contains("Thanks, you are on the list.") && !done.contains("<form"));
assert_eq!(subscribe(&Ui::default()).location(), "/?subscribed=1");
```

In an Axum app the routes take `ui: Ui` as an extractor and return the `Page` or `Redirect`
directly. The crate that holds this code needs `maud` as a dependency of its own, next to
`axum-nojs`: `html!` expands to `maud::` paths even when it comes from the prelude.

## The rules the library follows

These are the conventions of `CLAUDE.md`, restated for a component outside the crate.

- **One builder, reached from `ui`.** An extension trait (`SubscribeExt`) is the way to add a
  method to `Ui` from your crate; required arguments go in the call, everything else is a
  chained setter named after what it sets. No options structs.
- **Read input from `ui`.** `ui.param(..)`, `ui.params(..)` and `ui.state` hold the request;
  the route should not have to pass what the component can read.
- **Build from the primitives.** A visible button is `ui.button`, a field is `ui.input`, a
  box is `ui.card`; your CSS arranges them and styles your own parts by class. Never style a
  bare `button` or `input`: the library's own look comes from `button.rs` and `input.rs`,
  and yours should change with it.
- **Tokens only.** Colours, radii, shadows and spacing come from `--nojs-*` custom properties
  (`docs/theming.md`), so your component follows the theme, dark mode and a second palette.
- **Your own prefix.** `nojs-` belongs to the library; pick one for your classes and ids
  (`acme-`), and derive ids the caller does not care about with `axum_nojs::slug`.
- **CSS once per page.** `Page::css(CONST)` inlines it in the `<head>` after the library's
  stylesheet, minified, however many times the component appears.
- **One variant per request.** When the markup depends on the browser, branch on
  `ui.caps.has(Cap::X)` and emit only that variant (see `docs/caps.md`).
- **Swap roots update in place.** An element with an `id` and `data-nojs="swap"` is replaced
  by the enhancement script after a post or link inside it (`enhance::swap_id` builds the id);
  without the script the same markup is a normal navigation.
- **State in the URL or a cookie.** A mutation is a `<form method="post">` answered by a
  redirect; `Saved<T>` keeps a value per visitor in a cookie, and `T` must be a struct with
  named fields (a newtype like `Visitor(String)` does not encode and is silently not saved).
- **Test it without script.** `axum-nojs-test` renders a route through Blitz, which has no
  script engine: `Page::render(router, "/", "").await` then `is_visible`, `bbox`, `screenshot`.
