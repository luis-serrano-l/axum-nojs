//! Your own: a page for the pricing card, the component written in the demo crate (`pricing.rs`).

use crate::pricing::{PRICING_CSS, PricingExt};
use crate::site::page;
use axum::{Router, routing::get};
use loco_ui::prelude::*;

pub(crate) fn routes() -> Router {
    Router::new().route("/pricing", get(pricing_page))
}

/// Each page's live component, which the index shows too (`site::preview`); the index adds
/// [`PRICING_CSS`] as this page does.
pub(crate) const PREVIEWS: &[super::Preview] = &[("/pricing", pricing)];

/// Three tiers from `pricing.rs`, a component written outside the library; monthly or yearly
/// is a link that changes one parameter.
fn pricing(ui: &Ui) -> Markup {
    let yearly = ui.param("billing") == Some("yearly");
    let (period, [hobby, pro, team]) = if yearly {
        ("/year", ["$0", "$120", "$480"])
    } else {
        ("/month", ["$0", "$12", "$48"])
    };
    let (by_month, by_year) = (
        ui.link_without("billing"),
        ui.link_with("billing", "yearly"),
    );
    let pick = |text, href, on| {
        let b = ui.link_button(text, href).small().current(on);
        if on { b } else { b.ghost() }
    };
    lui! { Stack gap=6 {
        Cluster gap=1 { (pick("Monthly", &by_month, !yearly)) (pick("Yearly", &by_year, yearly)) }
        // code: /pricing
        Grid("14rem") {
            PricingCard("Hobby", hobby) period=(period) blurb="For a side project." {
                feature "1 project"; feature "Community support"; cta "Start free" "/pricing";
            }
            PricingCard("Pro", pro) period=(period) blurb="For a small team shipping weekly." featured {
                feature "10 projects"; feature "Email support"; feature "Custom domain"; cta "Upgrade to Pro" "/pricing";
            }
            PricingCard("Team", team) period=(period) blurb="For a company." {
                feature "Unlimited projects"; feature "SSO"; feature "Audit log"; cta "Talk to us" "/pricing";
            }
        }
        // end code
    } }
}

async fn pricing_page(ui: Ui) -> Page {
    page(&ui, "Pricing card", lui! { Stack gap=6 {
        (pricing(&ui))
        p class="lui-note" { (crate::site::note("The card lives in `demo/src/pricing.rs`: an extension trait on `Ui`, a builder, `impl Render` and a CSS const added with `Page::css`. See `docs/components.md`.")) }
    } }).css(PRICING_CSS)
}
