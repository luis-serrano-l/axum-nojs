# Draft: a Linear / Magic UI look for Loco, HTML first, script optional

*A draft for the owner to edit and post (r/rust, This Week in Rust, a blog). Nothing here has
been published.*

---

**loco-ui** is a server-rendered UI kit for Loco and Axum: buttons, forms, dialogs, menus, tabs, data tables,
a calendar and date picker, uploads and a kanban board written in Maud, in a Linear / Magic UI look.
Every one of them works with JavaScript turned off; one optional script only makes updates land in place.

That last part is the point, and it is tested rather than just claimed. CI renders every demo
route through [Blitz](https://github.com/DioxusLabs/blitz), a browser engine with no script
engine at all, and asserts layout on the result. A test allows exactly one optional script per
page and fails on anything inline. Every page is served under a strict Content-Security-Policy
(`script-src 'self'`, or `'none'` for a page without the script).

## How it works without script

The HTML platform does more than it used to: `<dialog>` and invoker commands, `popover`,
`<details name>` for tabs and accordions, anchor positioning, `:has()`, view transitions.
What the platform cannot do is a form post answered by a redirect (Post/Redirect/Get). State
lives in URLs and cookies, so every screen can be bookmarked and `curl`ed.

The server knows which browser it is talking to without script. Tiny `@supports` beacons set
one cookie per feature, and each component then sends only the variant that browser can use,
with no polyfills.

One optional 3.6 KB (gzipped) script makes the same markup update in place instead of
reloading. Block it and every page still works.

## What a page weighs

A demo page is 12–13.5 KB gzipped with the whole stylesheet inlined. No script is required.

## Writing your own

Components are layered: primitives (button, input, card, badge, icon, layout), components
built only from them, widgets built from those, and yours. A component of your own is an
extension trait on `Ui`, a builder and a CSS const, about 30 lines. The guide's code runs as
a doctest.

## When not to use it

For an app that reacts on every keystroke, such as an editor, a live canvas or real-time
collaboration, use Leptos or Dioxus. This kit is for admin panels, dashboards, settings,
forms, content sites and public services: things that refresh per action and must work
everywhere.

Repository: https://github.com/luis-serrano-l/loco-ui · Demo: *(link once hosted)*
