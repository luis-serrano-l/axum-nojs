//! # Chart
//!
//! A bar chart, a line chart or a sparkline drawn on the server as inline SVG: one series of
//! labelled values, a y axis rounded to readable steps, and the numbers themselves in a
//! visually hidden table for anyone who cannot see the picture. No chart library and no
//! script, where most kits need both.
//!
//! **Platform features:**
//! - Inline `<svg>` in HTML (Chrome 7, Firefox 4, Safari 5.1) with `role="img"`, named by its
//!   `<title>` and `<desc>` through `aria-labelledby`.
//! - A `<title>` inside each bar and point: the browser shows the value as a tooltip on hover.
//! - CSS custom properties in SVG `fill` and `stroke` (Chrome 49, Firefox 31, Safari 9.1): the
//!   colours are `--lui-*` tokens, so a chart follows the theme and dark mode.
//!
//! **Accessibility:** the SVG is `role="img"` named by the chart's title and description; the
//! data is also a table (row headers are the labels) hidden visually but read by screen
//! readers; a sparkline, which sits in a sentence, lists its values as hidden text instead. Checked by axe-core in headless Firefox on every demo route, both capability
//! variants, light and dark (no serious or critical violation).
//!
//! **What it does not do without script:** zoom, pan, or a crosshair that follows the pointer;
//! the value of a bar or point shows as the browser's own tooltip.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.chart("Signups").point("Mon", 12.0).point("Tue", 18.0).point("Wed", 9.0);
//! let html = m.render().into_string();
//! assert!(html.contains(r#"role="img""#) && html.contains("<title id=\"chart-signups-title\">Signups</title>"));
//! assert!(html.contains(r#"<th scope="row">Tue</th><td>18</td>"#));
//! // Values from data in one call:
//! const SIGNUPS: [(&str, f64); 3] = [("Mon", 12.0), ("Tue", 18.0), ("Wed", 9.0)];
//! assert_eq!(ui.chart("Signups").points(SIGNUPS).render().into_string(), html);
//!
//! let m = ui.chart("Latency").line().unit(" ms").description("p50 per day, last week")
//!     .point("Mon", 41.5).point("Tue", 38.0);
//! let html = m.render().into_string();
//! assert!(html.contains("lui-chart-line") && html.contains("<td>41.5 ms</td>"));
//! // The same in `lui!`:
//! let same = lui! { Chart("Latency") line unit=" ms" description="p50 per day, last week" {
//!     point "Mon" 41.5; point "Tue" 38.0;
//! } };
//! assert_eq!(same.into_string(), html);
//! ```

use std::fmt::Write;

use maud::{Markup, PreEscaped, Render, html};

use crate::props::{Prop, PropKind};
use crate::{Ui, slug};

/// Bars, a line or a sparkline, made by [`Ui::chart`].
///
/// **Setters.** Values and items: `.point(..)`, `.points(..)`, `.description(..)`, `.unit(..)`, `.id(..)`;
/// switches: `.bar()`, `.line()`, `.sparkline()`.
#[derive(Clone, Debug)]
pub struct Chart<'a> {
    title: &'a str,
    description: Option<&'a str>,
    unit: &'a str,
    kind: Kind,
    points: Vec<(&'a str, f64)>,
    id: Option<&'a str>,
}

/// Which drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Bar,
    Line,
    Sparkline,
}

impl Chart<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("point", PropKind::Item, "label: &'a str, value: f64")
            .doc("One value, labelled on the x axis."),
        Prop::new(
            "points",
            PropKind::Value,
            "points: impl IntoIterator<Item = (&'a str, f64)>",
        )
        .doc("Many values at once, from a slice or a query: `(label, value)` pairs."),
        Prop::new("description", PropKind::Value, "text: &'a str")
            .doc("What the chart shows, in a sentence: its `<desc>`."),
        Prop::new("unit", PropKind::Value, "unit: &'a str")
            .doc("Written after every value, as `\" ms\"` or `\"%\"`."),
        Prop::new("id", PropKind::Value, "id: &'a str")
            .attr("id")
            .doc("The id its parts are named from; `chart-<title>` by default."),
        Prop::new("bar", PropKind::Switch, "").doc("Bars, one per point (the default)."),
        Prop::new("line", PropKind::Switch, "").doc("A line through the points."),
        Prop::new("sparkline", PropKind::Switch, "")
            .doc("A small line with no axes, to sit beside a number."),
    ];
}

impl Ui {
    /// A chart titled `title`; add values with [`Chart::point`].
    pub fn chart<'a>(&self, title: &'a str) -> Chart<'a> {
        Chart {
            title,
            description: None,
            unit: "",
            kind: Kind::Bar,
            points: Vec::new(),
            id: None,
        }
    }
}

impl<'a> Chart<'a> {
    /// One value, labelled on the x axis.
    pub fn point(mut self, label: &'a str, value: f64) -> Self {
        self.points.push((label, value));
        self
    }

    /// Many values at once, from a slice or a query: `(label, value)` pairs, after any
    /// `.point(..)` added before.
    pub fn points(mut self, points: impl IntoIterator<Item = (&'a str, f64)>) -> Self {
        self.points.extend(points);
        self
    }

    /// What the chart shows, in a sentence: its `<desc>`.
    pub fn description(mut self, text: &'a str) -> Self {
        self.description = Some(text);
        self
    }

    /// Written after every value, as `" ms"` or `"%"`.
    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = unit;
        self
    }

    /// The id its parts are named from; `chart-<title>` by default.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    /// Bars, one per point (the default).
    pub fn bar(mut self) -> Self {
        self.kind = Kind::Bar;
        self
    }

    /// A line through the points.
    pub fn line(mut self) -> Self {
        self.kind = Kind::Line;
        self
    }

    /// A small line with no axes, to sit beside a number.
    pub fn sparkline(mut self) -> Self {
        self.kind = Kind::Sparkline;
        self
    }
}

/// A round step at or above `v`: 1, 2 or 5 times a power of ten.
fn nice(v: f64) -> f64 {
    if v <= 0.0 {
        return 1.0;
    }
    let power = 10f64.powf(v.log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .iter()
        .map(|m| m * power)
        .find(|n| *n >= v - 1e-9)
        .unwrap_or(10.0 * power)
}

/// The axis for values from `min` to `max`: `(low, high, step)`, zero included, at most
/// about five round steps.
fn axis(min: f64, max: f64) -> (f64, f64, f64) {
    let (min, max) = (min.min(0.0), max.max(0.0));
    let step = nice((max - min) / 4.0);
    let (lo, hi) = ((min / step).floor() * step, (max / step).ceil() * step);
    if (hi - lo).abs() < f64::EPSILON {
        (0.0, step, step)
    } else {
        (lo, hi, step)
    }
}

/// `12`, `12.5`, `0.25`: no trailing zeros.
fn number(v: f64) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Width and height of the drawing, in SVG units; the SVG scales to its box.
const W: f64 = 600.0;
const H: f64 = 240.0;
/// Room for the y labels on the left, the x labels below, and a little at the top.
const LEFT: f64 = 44.0;
const BOTTOM: f64 = 28.0;
const TOP: f64 = 10.0;

impl Render for Chart<'_> {
    fn render(&self) -> Markup {
        let id = self
            .id
            .map_or_else(|| format!("chart-{}", slug(self.title)), str::to_string);
        let (title_id, desc_id) = (format!("{id}-title"), format!("{id}-desc"));
        let labelled = if self.description.is_some() {
            format!("{title_id} {desc_id}")
        } else {
            title_id.clone()
        };
        let lo_value = self.points.iter().map(|p| p.1).fold(0.0, f64::min);
        let hi_value = self.points.iter().map(|p| p.1).fold(0.0, f64::max);
        let (lo, hi, tick) = axis(lo_value, hi_value);
        let sparkline = self.kind == Kind::Sparkline;
        let (w, h) = if sparkline { (120.0, 32.0) } else { (W, H) };
        let (left, bottom, top) = if sparkline {
            (2.0, 2.0, 2.0)
        } else {
            (LEFT, BOTTOM, TOP)
        };
        let plot = (w - left - 2.0, h - bottom - top);
        let y = |v: f64| top + (hi - v) / (hi - lo) * plot.1;
        let n = self.points.len().max(1) as f64;
        let step = plot.0 / n;
        let x = |i: usize| left + step * (i as f64 + 0.5);
        let value = |v: f64| format!("{}{}", number(v), self.unit);

        let mut marks = String::new();
        if !sparkline {
            let ticks = ((hi - lo) / tick).round() as u32;
            for t in 0..=ticks {
                let v = lo + tick * f64::from(t);
                let _ = write!(
                    marks,
                    r#"<line class="lui-chart-grid" stroke="currentColor" stroke-opacity="0.15" x1="{left}" x2="{}" y1="{y:.1}" y2="{y:.1}"/><text class="lui-chart-tick" font-family="sans-serif" font-size="12" x="{}" y="{:.1}" text-anchor="end">{}</text>"#,
                    w - 2.0,
                    left - 6.0,
                    y(v) + 4.0,
                    number(v),
                    y = y(v),
                );
            }
        }
        match self.kind {
            Kind::Bar => {
                let width = (step * 0.6).min(48.0);
                for (i, (label, v)) in self.points.iter().enumerate() {
                    let (a, b) = (y(v.max(0.0)), y(v.min(0.0)));
                    let _ = write!(
                        marks,
                        r#"<rect class="lui-chart-bar" x="{:.1}" y="{a:.1}" width="{width:.1}" height="{:.1}" rx="3"><title>{}: {}</title></rect>"#,
                        x(i) - width / 2.0,
                        (b - a).max(0.5),
                        escape(label),
                        escape(&value(*v)),
                    );
                }
            }
            Kind::Line | Kind::Sparkline => {
                let path: Vec<String> = self
                    .points
                    .iter()
                    .enumerate()
                    .map(|(i, (_, v))| format!("{:.1},{:.1}", x(i), y(*v)))
                    .collect();
                let _ = write!(
                    marks,
                    r#"<polyline class="lui-chart-path" fill="none" stroke="currentColor" stroke-width="2" points="{}"/>"#,
                    path.join(" ")
                );
                if !sparkline {
                    for (i, (label, v)) in self.points.iter().enumerate() {
                        let _ = write!(
                            marks,
                            r#"<circle class="lui-chart-dot" fill="none" stroke="currentColor" stroke-width="2" cx="{:.1}" cy="{:.1}" r="4"><title>{}: {}</title></circle>"#,
                            x(i),
                            y(*v),
                            escape(label),
                            escape(&value(*v)),
                        );
                    }
                }
            }
        }
        if !sparkline {
            for (i, (label, _)) in self.points.iter().enumerate() {
                let _ = write!(
                    marks,
                    r#"<text class="lui-chart-label" font-family="sans-serif" font-size="12" x="{:.1}" y="{:.1}" text-anchor="middle">{}</text>"#,
                    x(i),
                    h - 8.0,
                    escape(label)
                );
            }
        }
        let kind = match self.kind {
            Kind::Bar => "lui-chart lui-chart-bars",
            Kind::Line => "lui-chart lui-chart-line",
            Kind::Sparkline => "lui-chart lui-chart-sparkline",
        };
        let svg = html! {
            svg viewBox={ "0 0 " (w) " " (h) } role="img" aria-labelledby=(labelled) {
                title id=(title_id) { (self.title) }
                @if let Some(d) = self.description { desc id=(desc_id) { (d) } }
                (PreEscaped(marks))
            }
        };
        let table = html! {
            table {
                caption { (self.title) }
                @for (label, v) in &self.points { tr { th scope="row" { (label) } td { (value(*v)) } } }
            }
        };
        // A sparkline sits in a sentence, so it is phrasing content: a span, not a figure.
        if sparkline {
            html! { span class=(kind) id=(id) { (svg) span class="lui-sr" { (self.points.iter().map(|(l, v)| format!("{l}: {}", value(*v))).collect::<Vec<_>>().join(", ")) } } }
        } else {
            html! { figure class=(kind) id=(id) { (svg) div class="lui-sr" { (table) } } }
        }
    }
}

/// Text for inside an SVG element written by hand.
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn's charts: the primary
/// colour, a light grid, muted small labels. The marks also carry presentation attributes
/// (`fill="none"`, `currentColor`) for a renderer that draws the SVG without the page's CSS;
/// author CSS wins over them in a browser.
pub const CSS: &str = r#"
.lui-chart { margin: 0; }
figure.lui-chart { margin-block: calc(var(--lui-space) * 3); }
.lui-chart svg { display: block; width: 100%; height: auto; overflow: visible; }
.lui-chart-grid { stroke: var(--lui-line); stroke-width: 1; }
.lui-chart-tick, .lui-chart-label { fill: var(--lui-muted); font-size: 12px; font-family: inherit; }
.lui-chart-bar { fill: var(--lui-primary); }
.lui-chart-bar:hover { fill: color-mix(in srgb, var(--lui-primary) 80%, var(--lui-bg)); }
.lui-chart-path { fill: none; stroke: var(--lui-primary); stroke-width: 2; stroke-linejoin: round; stroke-linecap: round; vector-effect: non-scaling-stroke; }
.lui-chart-dot { fill: var(--lui-bg); stroke: var(--lui-primary); stroke-width: 2; }
.lui-chart-sparkline { display: inline-block; width: 7.5rem; vertical-align: middle; }
.lui-chart-sparkline svg { height: 2rem; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_axis_rounds_up_to_a_readable_step() {
        assert_eq!(axis(0.0, 22.0), (0.0, 30.0, 10.0));
        assert_eq!(axis(0.0, 18.0), (0.0, 20.0, 5.0));
        assert_eq!(axis(-3.0, 5.0), (-4.0, 6.0, 2.0));
        assert_eq!(axis(0.0, 0.0), (0.0, 1.0, 1.0));
        assert_eq!(number(12.50), "12.5");
        assert_eq!(number(3.0), "3");
    }

    #[test]
    fn negative_values_hang_below_zero() {
        let m = Ui::default()
            .chart("Profit")
            .point("Q1", 5.0)
            .point("Q2", -3.0)
            .render()
            .into_string();
        assert!(m.contains(">-4</text>") && m.contains("Q2: -3"), "{m}");
    }
}
