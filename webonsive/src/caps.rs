//! # Caps
//!
//! Server-side feature detection with no script. The page carries a few empty beacon elements;
//! `@supports` rules give each one a background image only when the browser understands the
//! feature. Loading that image hits `/wo/caps?flag=<name>`, which sets a cookie. From the next
//! request on, the server knows what the browser can do and every component emits only the
//! markup that browser needs.
//!
//! **Platform features:** `@supports` (baseline 2015), `@supports selector()` (baseline 2022),
//! CSS background images, cookies. Nothing else.
//!
//! **Fallback:** an unknown browser has no cookie and gets the fallback markup everywhere. The
//! first page view always renders as a fallback: the beacons fire while it loads, and the second
//! view is tailored. Browsers that never load CSS images (`curl`, readers) stay on fallbacks.
//!
//! **Finding:** CSS cannot test HTML attributes, so `invokers` and `streaming_dsd` are proxies
//! for CSS features that shipped in the same release. See `FINDINGS.md`.
//!
//! One cookie per flag (`wo-cap-<name>=1`) rather than one cookie holding a list: the beacons
//! fire in parallel, and parallel `Set-Cookie` headers on one name would overwrite each other.
//!
//! ```rust
//! use webonsive::caps::{Cap, Caps};
//! let caps = Caps::from_cookie_header("wo-cap-probed=1; wo-cap-popover=1; theme=dark");
//! assert!(caps.has(Cap::Popover));
//! assert!(!caps.has(Cap::Invokers));
//! assert_eq!(Caps::all().names().len(), Cap::ALL.len());
//! ```

use maud::{Markup, html};

/// Path of the beacon route. `beacon_css` builds URLs from it; `router` serves it.
pub const BEACON_PATH: &str = "/wo/caps";

/// Cookie name prefix: the flag `popover` lives in the cookie `wo-cap-popover`.
pub const COOKIE_PREFIX: &str = "wo-cap-";

/// Cookie lifetime in seconds (30 days), so browser upgrades get re-detected eventually.
pub const COOKIE_MAX_AGE: u32 = 30 * 24 * 60 * 60;

/// One browser capability the server can learn about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cap {
    /// The browser loaded the beacons at all. Absent on a first visit or a non-visual client.
    Probed,
    /// `<button command commandfor>` invoker commands (Chrome 135, Firefox 144, Safari 26.2).
    Invokers,
    /// CSS anchor positioning (Chrome 125, Firefox 147, Safari 26).
    Anchor,
    /// `::details-content` pseudo-element (Chrome 131, Firefox 143, Safari 18.4).
    DetailsContent,
    /// View transitions: `view-transition-name` and `-class` (Chrome 125, Firefox 144, Safari 18.2).
    ViewTransitions,
    /// `popover` attribute and `:popover-open` (Chrome 114, Firefox 125, Safari 17).
    Popover,
    /// `light-dark()` colour function (Chrome 123, Firefox 120, Safari 17.5).
    LightDark,
    /// Declarative shadow DOM, `<template shadowrootmode>` (Chrome 111, Firefox 123, Safari 16.4).
    StreamingDsd,
}

impl Cap {
    /// Every capability, in display order.
    pub const ALL: [Cap; 8] = [
        Cap::Probed,
        Cap::Invokers,
        Cap::Anchor,
        Cap::DetailsContent,
        Cap::ViewTransitions,
        Cap::Popover,
        Cap::LightDark,
        Cap::StreamingDsd,
    ];

    /// Flag name used in cookies, beacon URLs and CSS class names.
    pub fn name(self) -> &'static str {
        match self {
            Cap::Probed => "probed",
            Cap::Invokers => "invokers",
            Cap::Anchor => "anchor",
            Cap::DetailsContent => "details_content",
            Cap::ViewTransitions => "view_transitions",
            Cap::Popover => "popover",
            Cap::LightDark => "light_dark",
            Cap::StreamingDsd => "streaming_dsd",
        }
    }

    /// Parse a flag name. Unknown names give `None`.
    pub fn parse(name: &str) -> Option<Cap> {
        Cap::ALL.into_iter().find(|c| c.name() == name)
    }

    /// The `@supports` condition that detects this capability, or `None` when the beacon fires
    /// unconditionally. Proxies are noted in the doc header and in `FINDINGS.md`.
    pub fn supports(self) -> Option<&'static str> {
        match self {
            Cap::Probed => None,
            // No CSS test exists for HTML attributes. `::picker(select)` shipped with invokers
            // in Chrome 135 (Safari: 27); Firefox 144 shipped `view-transition-class` with them.
            Cap::Invokers => Some(
                "selector(::picker(select)) or ((-moz-appearance: none) and (view-transition-class: x))",
            ),
            Cap::Anchor => Some("(anchor-name: --x)"),
            Cap::DetailsContent => Some("selector(::details-content)"),
            Cap::ViewTransitions => Some("(view-transition-class: x)"),
            Cap::Popover => Some("selector(:popover-open)"),
            Cap::LightDark => Some("(color: light-dark(#000, #fff))"),
            // Proxy: `:popover-open` shipped after declarative shadow DOM in every engine.
            Cap::StreamingDsd => Some("selector(:popover-open)"),
        }
    }

    /// One line for humans and the `/caps` page.
    pub fn description(self) -> &'static str {
        match self {
            Cap::Probed => {
                "beacons loaded at all (set on every visual browser after the first view)"
            }
            Cap::Invokers => {
                "dialog opens with <button command=show-modal> instead of a :target link"
            }
            Cap::Anchor => "popover menus sit under their button via anchor positioning",
            Cap::DetailsContent => {
                "tabs lay out as a strip via ::details-content; otherwise an accordion"
            }
            Cap::ViewTransitions => {
                "counter and list get view-transition-name for morphing navigations"
            }
            Cap::Popover => "menus use the popover attribute; otherwise a <details> dropdown",
            Cap::LightDark => {
                "light-dark() is understood (informational; theming uses media queries)"
            }
            Cap::StreamingDsd => "out-of-order streaming via declarative shadow DOM slots",
        }
    }

    fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// The set of capabilities the server believes the current browser has.
///
/// `Caps::default()` is empty: every component renders its fallback. `Caps::all()` is the
/// modern-browser view used in doc examples and tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Caps(u16);

impl Caps {
    /// Nothing known: fallback markup everywhere.
    pub const NONE: Caps = Caps(0);

    /// Every capability, for examples and tests.
    pub fn all() -> Caps {
        Cap::ALL.into_iter().fold(Caps::NONE, Caps::with)
    }

    /// Whether `cap` is known to be supported.
    pub fn has(self, cap: Cap) -> bool {
        self.0 & cap.bit() != 0
    }

    /// Copy with `cap` added.
    pub fn with(self, cap: Cap) -> Caps {
        Caps(self.0 | cap.bit())
    }

    /// Names of every supported capability, in `Cap::ALL` order.
    pub fn names(self) -> Vec<&'static str> {
        Cap::ALL
            .into_iter()
            .filter(|c| self.has(*c))
            .map(Cap::name)
            .collect()
    }

    /// Read the flags out of a raw `Cookie:` header value.
    pub fn from_cookie_header(header: &str) -> Caps {
        header
            .split(';')
            .filter_map(|pair| pair.trim().split_once('='))
            .filter(|(_, value)| value.trim() == "1")
            .filter_map(|(name, _)| name.trim().strip_prefix(COOKIE_PREFIX))
            .filter_map(Cap::parse)
            .fold(Caps::NONE, Caps::with)
    }
}

/// The `@supports` rules. Each one gives a beacon element a background image whose URL is the
/// beacon route with the flag name. Included in `stylesheet()`.
pub fn beacon_css() -> String {
    let mut css = String::from(
        "\n.wo-caps { position: fixed; bottom: 0; right: 0; width: 1px; height: 1px; overflow: hidden; opacity: 0; pointer-events: none; }\n.wo-cap { display: block; width: 1px; height: 1px; }\n",
    );
    for cap in Cap::ALL {
        let rule = format!(
            ".wo-cap-{n} {{ background-image: url(\"{BEACON_PATH}?flag={n}\"); }}",
            n = cap.name()
        );
        match cap.supports() {
            Some(test) => css.push_str(&format!("@supports {test} {{ {rule} }}\n")),
            None => {
                css.push_str(&rule);
                css.push('\n');
            }
        }
    }
    css
}

/// The beacon elements. Empty once the browser has been probed, so a known browser pays
/// nothing. `layout` puts this at the end of `<body>`.
pub fn beacons(caps: &Caps) -> Markup {
    html! {
        @if !caps.has(Cap::Probed) {
            div class="wo-caps" aria-hidden="true" {
                @for cap in Cap::ALL {
                    i class={ "wo-cap wo-cap-" (cap.name()) } {}
                }
            }
        }
    }
}

/// `Set-Cookie` value that records `cap` for 30 days.
pub fn cookie_for(cap: Cap) -> String {
    format!(
        "{COOKIE_PREFIX}{}=1; Path=/; Max-Age={COOKIE_MAX_AGE}; SameSite=Lax",
        cap.name()
    )
}

#[cfg(feature = "axum")]
mod axum_glue {
    use super::{BEACON_PATH, Cap, Caps, cookie_for};
    use axum::{
        Router,
        extract::{FromRequestParts, Query},
        http::{HeaderMap, StatusCode, header, request::Parts},
        routing::get,
    };
    use std::collections::HashMap;

    impl<S: Send + Sync> FromRequestParts<S> for Caps {
        type Rejection = std::convert::Infallible;

        async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Caps, Self::Rejection> {
            let caps = parts
                .headers
                .get_all(header::COOKIE)
                .iter()
                .filter_map(|v| v.to_str().ok())
                .map(Caps::from_cookie_header)
                .fold(Caps::NONE, |acc, c| Caps(acc.0 | c.0));
            Ok(caps)
        }
    }

    /// Serves `GET /wo/caps?flag=<name>`: sets the flag's cookie and answers `204`.
    /// Merge it into your app: `Router::new().merge(webonsive::caps::router())`.
    pub fn router() -> Router {
        Router::new().route(BEACON_PATH, get(beacon))
    }

    async fn beacon(Query(q): Query<HashMap<String, String>>) -> (StatusCode, HeaderMap) {
        let mut headers = HeaderMap::new();
        headers.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
        match q.get("flag").and_then(|f| Cap::parse(f)) {
            Some(cap) => {
                headers.insert(header::SET_COOKIE, cookie_for(cap).parse().unwrap());
                (StatusCode::NO_CONTENT, headers)
            }
            None => (StatusCode::NOT_FOUND, headers),
        }
    }
}

#[cfg(feature = "axum")]
pub use axum_glue::router;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_round_trip() {
        let header = Cap::ALL
            .map(|c| cookie_for(c).split(';').next().unwrap().to_string())
            .join("; ");
        assert_eq!(Caps::from_cookie_header(&header), Caps::all());
        assert_eq!(
            Caps::from_cookie_header("wo-cap-bogus=1; wo-cap-popover=0"),
            Caps::NONE
        );
    }

    #[test]
    fn beacons_disappear_once_probed() {
        assert!(
            beacons(&Caps::NONE)
                .into_string()
                .contains("wo-cap-invokers")
        );
        assert_eq!(beacons(&Caps::NONE.with(Cap::Probed)).into_string(), "");
    }

    #[test]
    fn css_has_one_rule_per_cap() {
        let css = beacon_css();
        for cap in Cap::ALL {
            assert!(
                css.contains(&format!("?flag={}", cap.name())),
                "{}",
                cap.name()
            );
        }
    }
}
