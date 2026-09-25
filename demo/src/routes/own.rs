//! Your own: a page for the pricing card, the component written in the demo crate (`pricing.rs`).

use crate::pricing::{PRICING_CSS, PricingExt};
use crate::site::page;
use axum::{Router, routing::get};
use axum_nojs::prelude::*;

pub(crate) fn routes() -> Router {
    Router::new().route("/pricing", get(pricing_page))
}

/// Three tiers from `pricing.rs`, a component written outside the library; monthly or yearly
/// is a link that changes one parameter.
async fn pricing_page(ui: Ui) -> Page {
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
    page(&ui, "Pricing card", html! { (ui.stack(html! {
        (ui.cluster(html! { (pick("Monthly", &by_month, !yearly)) (pick("Yearly", &by_year, yearly)) }).gap(1))
        // code: /pricing
        (ui.grid("14rem", html! {
            (ui.pricing_card("Hobby", hobby).period(period).blurb("For a side project.")
                .feature("1 project").feature("Community support").cta("Start free", "/pricing"))
            (ui.pricing_card("Pro", pro).period(period).blurb("For a small team shipping weekly.").featured()
                .feature("10 projects").feature("Email support").feature("Custom domain").cta("Upgrade to Pro", "/pricing"))
            (ui.pricing_card("Team", team).period(period).blurb("For a company.")
                .feature("Unlimited projects").feature("SSO").feature("Audit log").cta("Talk to us", "/pricing"))
        }))
        // end code
        p class="nojs-note" { "The card lives in " code { "demo/src/pricing.rs" } ": an extension trait on " code { "Ui" } ", a builder, " code { "impl Render" } " and a CSS const added with " code { "Page::css" } ". See " code { "docs/components.md" } "." }
    }).gap(6)) }).css(PRICING_CSS)
}
