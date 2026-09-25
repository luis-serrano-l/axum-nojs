//! A component written outside the library, the way `docs/components.md` describes: an
//! extension trait on `Ui`, a builder holding `&Ui`, `impl Render`, and a CSS const the page
//! adds with `Page::css`. It uses only public `loco-ui` API (primitives and tokens), so it
//! restyles with the rest when the theme or a primitive changes.

use loco_ui::{Icon, prelude::*, slug};

/// A pricing tier: name, price, what it includes, and a call to action.
pub struct PricingCard<'a> {
    ui: &'a Ui,
    name: &'a str,
    price: &'a str,
    period: &'a str,
    blurb: Option<&'a str>,
    features: Vec<&'a str>,
    cta: (&'a str, &'a str),
    featured: bool,
}

/// `ui.pricing_card(..)`, next to the built-in components.
pub trait PricingExt {
    /// A tier called `name` at `price` (already formatted, `"$12"`).
    fn pricing_card<'a>(&'a self, name: &'a str, price: &'a str) -> PricingCard<'a>;
}

impl PricingExt for Ui {
    fn pricing_card<'a>(&'a self, name: &'a str, price: &'a str) -> PricingCard<'a> {
        PricingCard {
            ui: self,
            name,
            price,
            period: "/month",
            blurb: None,
            features: Vec::new(),
            cta: ("Choose plan", "#"),
            featured: false,
        }
    }
}

impl<'a> PricingCard<'a> {
    /// What the price is per (`"/year"`); `/month` by default.
    pub fn period(mut self, period: &'a str) -> Self {
        self.period = period;
        self
    }

    /// One line under the name: who the tier is for.
    pub fn blurb(mut self, text: &'a str) -> Self {
        self.blurb = Some(text);
        self
    }

    /// One thing the tier includes; call once per line.
    pub fn feature(mut self, text: &'a str) -> Self {
        self.features.push(text);
        self
    }

    /// The button: its text and where it goes.
    pub fn cta(mut self, text: &'a str, href: &'a str) -> Self {
        self.cta = (text, href);
        self
    }

    /// The tier to steer people to: a "Popular" badge, a stronger border, a primary button.
    pub fn featured(mut self) -> Self {
        self.featured = true;
        self
    }
}

impl Render for PricingCard<'_> {
    fn render(&self) -> Markup {
        let ui = self.ui;
        let (text, href) = self.cta;
        let button = ui.link_button(text, href);
        let button = if self.featured {
            button.primary()
        } else {
            button
        };
        let body = html! {
            (ui.stack(html! {
                p class="demo-pricing-price" { span { (self.price) } span class="demo-pricing-period" { (self.period) } }
                ul class="demo-pricing-features" {
                    @for f in &self.features { li { (Icon::Check) span { (f) } } }
                }
            }).gap(4))
        };
        let id = format!("demo-pricing-{}", slug(self.name));
        let card = ui
            .card()
            .id(&id)
            .title(self.name)
            .body(body)
            .footer(html! { (button.class("demo-pricing-cta")) });
        let card = match (self.blurb, self.featured) {
            (Some(b), true) => card.description(b).header(html! { (ui.badge("Popular")) }),
            (Some(b), false) => card.description(b),
            (None, true) => card.header(html! { (ui.badge("Popular")) }),
            (None, false) => card,
        };
        html! { div class={ "demo-pricing" @if self.featured { " demo-pricing-featured" } } { (card) } }
    }
}

/// The card's own styles: classes prefixed `demo-`, sizes and colours from `--lui-*` tokens.
pub const PRICING_CSS: &str = r#"
.demo-pricing, .demo-pricing > .lui-card { height: 100%; box-sizing: border-box; }
.demo-pricing-featured > .lui-card { border-color: var(--lui-primary); box-shadow: var(--lui-shadow-lg); }
.demo-pricing-price { display: flex; align-items: baseline; gap: 0.25rem; margin: 0; }
.demo-pricing-price > span:first-child { font-size: 2.25rem; line-height: 2.5rem; font-weight: 600; letter-spacing: -0.025em; }
.demo-pricing-period { color: var(--lui-muted); font-size: 0.875rem; }
.demo-pricing-features { list-style: none; margin: 0; padding: 0; display: grid; gap: 0.5rem; font-size: 0.875rem; }
.demo-pricing-features li { display: flex; align-items: center; gap: 0.5rem; }
.demo-pricing-features .lui-icon { color: var(--lui-ok); }
.demo-pricing .lui-card-body { flex: 1; }
.demo-pricing-cta { width: 100%; }
"#;
