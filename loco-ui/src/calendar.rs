//! # Calendar
//!
//! A month grid drawn on the server: previous and next month are links, each day is a link
//! (pick a day and the page comes back with it) or a radio button (the day is a form field,
//! for a date picker). Days before `.min()`, after `.max()` or ruled out by `.disabled(..)`
//! cannot be picked; `.event(..)` puts a dot under a day.
//!
//! **Platform features:** a `<table>` of links or `<input type="radio">` inside `<label>`s,
//! `aria-current="date"` on today, `role="radiogroup"` when the days are radios, `:has(:checked)`
//! (Chrome 105, Firefox 121, Safari 15.4) to draw the picked radio's day. The month shown
//! travels in `?month.<name>=YYYY-MM`, the picked day in `?<name>=YYYY-MM-DD`.
//!
//! **Accessibility:** a `<table>` with column headers (`abbr` holds the full weekday); each day
//! is a link or radio whose hidden text says the whole date, today is `aria-current="date"`,
//! the month title is `aria-live`; arrow keys are not wired (Tab moves through days). Checked
//! by axe-core in headless Firefox on every demo route, both capability variants, light and
//! dark (no serious or critical violation).
//!
//! **What it does not do without script:** change month without a page load (the enhancement
//! script swaps the calendar in place: it is a swap root), or move between days with the arrow
//! keys (Tab goes through them in order). In radio mode a month link leaves the page, so a
//! form's other unsaved fields are lost; put the calendar in its own form or step.
//!
//! **Fallback:** without `:has()` the picked radio's day is not filled in, but the radio is
//! still checked and posts.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from_request("/book", "month.due=2026-09", "");
//! let m = ui.calendar("due").today("2026-09-24").render().into_string();
//! assert!(m.contains("September 2026") && m.contains(r#"aria-current="date""#));
//! assert!(m.contains(r#"href="/book?month.due=2026-09&amp;due=2026-09-15""#));
//! // Radios for a form, a window of days, weekends off, a deadline marked, Sunday first.
//! let m = ui.calendar("due").radio().required()
//!     .min("2026-09-10").max("2026-10-31")
//!     .disabled(|d| d.weekday() >= 5)
//!     .event("2026-09-30", "Invoice due")
//!     .sunday_first();
//! let m = m.render().into_string();
//! assert!(m.contains(r#"type="radio" name="due" value="2026-09-11""#));
//! assert!(m.contains(r#"value="2026-09-12" disabled"#) && m.contains("Invoice due"));
//! // The same in `lui!`:
//! let same = lui! { Calendar("due") radio required min="2026-09-10" max="2026-10-31"
//!     disabled=(|d| d.weekday() >= 5) event=("2026-09-30", "Invoice due") sunday_first; };
//! assert_eq!(same.into_string(), m);
//! ```

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use maud::{Markup, Render, html};

use crate::button::Button;
use crate::i18n::{Strings, Text};
use crate::props::{Prop, PropKind};
use crate::{Icon, Ui, enhance};

/// A calendar day, `YYYY-MM-DD`, in the proleptic Gregorian calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    /// The year, e.g. 2026.
    pub year: i32,
    /// 1 to 12.
    pub month: u8,
    /// 1 to the month's length.
    pub day: u8,
}

impl Date {
    /// The date, or `None` when the day does not exist (`2026-02-30`).
    pub fn new(year: i32, month: u8, day: u8) -> Option<Date> {
        ((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month))
            .then_some(Date { year, month, day })
    }

    /// `YYYY-MM-DD`, as `<input type="date">` and the query string carry it.
    pub fn parse(s: &str) -> Option<Date> {
        let mut parts = s.trim().splitn(3, '-');
        let year = parts.next()?.parse().ok()?;
        let month = parts.next()?.parse().ok()?;
        let day = parts.next()?.parse().ok()?;
        Date::new(year, month, day)
    }

    /// Today in UTC, from the system clock.
    pub fn today() -> Date {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        Date::from_days((secs / 86_400) as i64)
    }

    /// 0 for Monday up to 6 for Sunday.
    pub fn weekday(self) -> u8 {
        // 1970-01-01 was a Thursday (3).
        (self.days() + 3).rem_euclid(7) as u8
    }

    /// The date `n` days later (earlier when negative).
    pub fn add_days(self, n: i64) -> Date {
        Date::from_days(self.days() + n)
    }

    /// The first day of the month `n` months later (earlier when negative).
    pub fn add_months(self, n: i32) -> Date {
        let index = self.year * 12 + i32::from(self.month) - 1 + n;
        Date {
            year: index.div_euclid(12),
            month: (index.rem_euclid(12) + 1) as u8,
            day: 1,
        }
    }

    /// Days since 1970-01-01 (Howard Hinnant's `days_from_civil`).
    fn days(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// The inverse of [`Date::days`] (`civil_from_days`).
    fn from_days(z: i64) -> Date {
        let z = z + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Date { year, month, day }
    }

    /// `"24 September 2026"`, the date as a button or a sentence shows it.
    pub(crate) fn long(self, s: &Strings) -> String {
        let month = s.get(Text::month(u32::from(self.month)));
        s.fill(Text::DayMonthYear, &[&self.day, &month, &self.year])
    }

    /// `"Thursday, 24 September 2026"`, what a screen reader says for the day.
    pub(crate) fn spoken(self, s: &Strings) -> String {
        let weekday = s.get(Text::weekday(u32::from(self.weekday())));
        let month = s.get(Text::month(u32::from(self.month)));
        s.fill(Text::LongDate, &[&weekday, &self.day, &month, &self.year])
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// `YYYY-MM` to the first of that month.
fn parse_month(s: &str) -> Option<Date> {
    let (y, m) = s.trim().split_once('-')?;
    Date::new(y.parse().ok()?, m.parse().ok()?, 1)
}

/// A month grid, made by [`Ui::calendar`].
///
/// **Setters.** Values and items: `.disabled(..)`, `.today(..)`, `.min(..)`, `.max(..)`,
/// `.event(..)`, `.value(..)`; switches: `.sunday_first()`, `.radio()`, `.required()`.
#[derive(Clone, Debug)]
pub struct Calendar<'a> {
    ui: &'a Ui,
    name: &'a str,
    today: Option<Date>,
    min: Option<Date>,
    max: Option<Date>,
    disabled: Option<fn(Date) -> bool>,
    events: Vec<(Date, &'a str)>,
    sunday_first: bool,
    radio: bool,
    required: bool,
    value: Option<Date>,
}

impl Calendar<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("today", PropKind::Value, "date: &str")
            .doc("The date to mark as today (`YYYY-MM-DD`), instead of the server's clock in UTC."),
        Prop::new("min", PropKind::Value, "date: &str")
            .doc("The first day that can be picked (`YYYY-MM-DD`)."),
        Prop::new("max", PropKind::Value, "date: &str")
            .doc("The last day that can be picked (`YYYY-MM-DD`)."),
        Prop::new("disabled", PropKind::Value, "off: fn(Date) -> bool").attr("disabled")
            .doc("Days for which `off` returns true cannot be picked."),
        Prop::new("event", PropKind::Item, "date: &str, text: &'a str")
            .doc("A dot under the day `date` (`YYYY-MM-DD`), with `text` for screen readers and as the day's tooltip."),
        Prop::new("sunday_first", PropKind::Switch, "")
            .doc("Weeks start on Sunday instead of Monday."),
        Prop::new("radio", PropKind::Switch, "")
            .doc("Each day is a radio button named after the calendar, to post inside a form, instead of a link."),
        Prop::new("value", PropKind::Value, "date: &str").attr("value")
            .doc("The picked day (`YYYY-MM-DD`) when it does not come from the query string."),
        Prop::new("required", PropKind::Switch, "").attr("required")
            .doc("In radio mode, a day must be picked before the form submits."),
    ];
}

impl Ui {
    /// A calendar whose picked day is the query parameter `name` (`?due=2026-09-24`) and whose
    /// month is `?month.<name>=2026-09`; the month of the picked day, or today's, otherwise.
    pub fn calendar<'a>(&'a self, name: &'a str) -> Calendar<'a> {
        Calendar {
            ui: self,
            name,
            today: None,
            min: None,
            max: None,
            disabled: None,
            events: Vec::new(),
            sunday_first: false,
            radio: false,
            required: false,
            value: None,
        }
    }
}

impl<'a> Calendar<'a> {
    /// The date to mark as today (`YYYY-MM-DD`), instead of the server's clock in UTC: the
    /// visitor's own day when the server knows their time zone.
    pub fn today(mut self, date: &str) -> Self {
        self.today = Date::parse(date);
        self
    }

    /// The first day that can be picked (`YYYY-MM-DD`); earlier months are not linked.
    pub fn min(mut self, date: &str) -> Self {
        self.min = Date::parse(date);
        self
    }

    /// The last day that can be picked (`YYYY-MM-DD`); later months are not linked.
    pub fn max(mut self, date: &str) -> Self {
        self.max = Date::parse(date);
        self
    }

    /// Days for which `off` returns true cannot be picked: weekends, holidays, full days.
    pub fn disabled(mut self, off: fn(Date) -> bool) -> Self {
        self.disabled = Some(off);
        self
    }

    /// A dot under the day `date` (`YYYY-MM-DD`), with `text` for screen readers and as the
    /// day's tooltip. Several on one day are allowed.
    pub fn event(mut self, date: &str, text: &'a str) -> Self {
        if let Some(d) = Date::parse(date) {
            self.events.push((d, text));
        }
        self
    }

    /// Weeks start on Sunday instead of Monday.
    pub fn sunday_first(mut self) -> Self {
        self.sunday_first = true;
        self
    }

    /// Each day is a radio button named after the calendar, to post inside a form, instead of
    /// a link.
    pub fn radio(mut self) -> Self {
        self.radio = true;
        self
    }

    /// The picked day (`YYYY-MM-DD`) when it does not come from the query string: a saved
    /// value, the form's own state.
    pub fn value(mut self, date: &str) -> Self {
        self.value = Date::parse(date);
        self
    }

    /// In radio mode, a day must be picked before the form submits.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    fn off(&self, d: Date) -> bool {
        self.min.is_some_and(|m| d < m)
            || self.max.is_some_and(|m| d > m)
            || self.disabled.is_some_and(|f| f(d))
    }
}

impl Render for Calendar<'_> {
    fn render(&self) -> Markup {
        let ui = self.ui;
        let name = self.name;
        let month_key = format!("month.{name}");
        let today = self.today.unwrap_or_else(Date::today);
        let picked = self
            .value
            .or_else(|| ui.param(name).and_then(Date::parse))
            .filter(|&d| !self.off(d));
        let shown = ui
            .param(&month_key)
            .and_then(parse_month)
            .or(picked.map(|d| d.add_months(0)))
            .unwrap_or(today.add_months(0));
        // Month links stop at the months that still hold a day to pick.
        let prev = shown.add_months(-1);
        let next = shown.add_months(1);
        let has_prev = self.min.is_none_or(|m| shown.add_days(-1) >= m);
        let has_next = self.max.is_none_or(|m| next <= m);
        let start = if self.sunday_first { 6 } else { 0 };
        let lead = (i64::from(shown.weekday()) - start).rem_euclid(7);
        let first = shown.add_days(-lead);
        let len = i64::from(days_in_month(shown.year, shown.month));
        let weeks = (lead + len + 6) / 7;
        let root = enhance::swap_id("lui-calendar", name);
        let title_id = format!("{root}-title");
        let month = ui.text(Text::month(u32::from(shown.month)));
        let title = ui.fill(Text::MonthYear, &[&month, &shown.year]);
        let (prev_href, next_href) = (
            ui.link_with(&month_key, &format!("{:04}-{:02}", prev.year, prev.month)),
            ui.link_with(&month_key, &format!("{:04}-{:02}", next.year, next.month)),
        );
        let nav = |href: &str, label: &'static str, icon: Icon, on: bool| -> Markup {
            let b = Button::link(ui.caps, "", href)
                .ghost()
                .small()
                .icon()
                .label(label)
                .content(html! { (icon) });
            if on {
                b.render()
            } else {
                b.disabled().render()
            }
        };
        let weekdays = (0..7).map(|i| ui.text(Text::weekday((i + start) as u32)));
        html! {
            div id=(root) data-lui="swap" class="lui-calendar"
                role=(if self.radio { "radiogroup" } else { "group" }) aria-labelledby=(title_id)
                aria-required=[(self.radio && self.required).then_some("true")] {
                div class="lui-calendar-head" {
                    (nav(&prev_href, ui.text(Text::PreviousMonth), Icon::ChevronLeft, has_prev))
                    p id=(title_id) class="lui-calendar-title" aria-live="polite" { (title) }
                    (nav(&next_href, ui.text(Text::NextMonth), Icon::ChevronRight, has_next))
                }
                table class="lui-calendar-grid" aria-labelledby=(title_id) {
                    thead { tr { @for w in weekdays { th scope="col" abbr=(w) { (w.chars().take(2).collect::<String>()) } } } }
                    tbody {
                        @for week in 0..weeks { tr {
                            @for i in 0..7 {
                                @let d = first.add_days(week * 7 + i);
                                (self.day(d, d.month != shown.month, d == today, picked == Some(d)))
                            }
                        } }
                    }
                }
            }
        }
    }
}

impl Calendar<'_> {
    fn day(&self, d: Date, outside: bool, today: bool, picked: bool) -> Markup {
        let off = self.off(d);
        let events: Vec<&str> = self
            .events
            .iter()
            .filter(|e| e.0 == d)
            .map(|e| e.1)
            .collect();
        let class = format!(
            "lui-calendar-day{}{}{}{}",
            if outside { " lui-calendar-outside" } else { "" },
            if today { " lui-calendar-today" } else { "" },
            if picked { " lui-calendar-picked" } else { "" },
            if off { " lui-calendar-off" } else { "" },
        );
        let tip = (!events.is_empty()).then(|| events.join(", "));
        let face = html! {
            span aria-hidden="true" { (d.day) }
            span class="lui-sr" {
                (d.spoken(self.ui.strings)) @for e in &events { ", " (e) }
                @if picked && !self.radio { (self.ui.text(Text::IsSelected)) }
            }
            @if !events.is_empty() {
                span class="lui-calendar-dots" aria-hidden="true" { @for _ in events.iter().take(3) { span {} } }
            }
        };
        let current = today.then_some("date");
        let value = d.to_string();
        html! {
            td {
                @if self.radio {
                    label class=(class) title=[tip] aria-current=[current] {
                        input class="lui-calendar-radio" type="radio" name=(self.name) value=(value)
                            checked[picked] disabled[off] required[self.required && !off];
                        (face)
                    }
                } @else if off {
                    span class=(class) aria-disabled="true" title=[tip] { (face) }
                } @else {
                    a class=(class) href=(self.ui.link_with(self.name, &value)) title=[tip]
                        aria-current=[current] { (face) }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn Calendar: p-3, 2rem
/// day cells, the picked day in the primary colour, today on the accent.
pub const CSS: &str = r#"
.lui-calendar {
  display: inline-block; padding: 0.75rem; background: var(--lui-card); color: var(--lui-fg);
  border: 1px solid var(--lui-line); border-radius: var(--lui-radius); box-shadow: var(--lui-shadow-xs);
}
.lui-calendar-head { display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; margin-bottom: 0.5rem; }
.lui-calendar-title { margin: 0; font-size: 0.875rem; font-weight: 500; }
.lui-calendar-head .lui-button[aria-disabled=true] { visibility: hidden; }
.lui-calendar-grid { width: auto; border-collapse: separate; border-spacing: 0 0.25rem; font-size: 0.875rem; }
.lui-calendar-grid th { height: auto; padding: 0 0 0.25rem; width: 2.25rem; text-align: center; font-size: 0.8rem; font-weight: 400; color: var(--lui-muted); border: 0; }
.lui-calendar-grid td { padding: 0; border: 0; text-align: center; }
.lui-calendar-grid tbody tr:hover { background: none; }
.lui-calendar-day {
  position: relative; display: inline-flex; flex-direction: column; align-items: center; justify-content: center;
  width: 2.25rem; height: 2.25rem; box-sizing: border-box; border-radius: var(--lui-radius-sm);
  color: inherit; text-decoration: none; font-weight: 400; cursor: pointer; font-variant-numeric: tabular-nums;
}
a.lui-calendar-day:hover, label.lui-calendar-day:not(.lui-calendar-off):hover { background: var(--lui-accent); color: var(--lui-on-accent); }
.lui-calendar-outside { color: var(--lui-muted); }
.lui-calendar-today { background: var(--lui-accent); color: var(--lui-on-accent); }
/* Two rules, not one list: a browser without :has() drops a whole selector list it cannot
   parse, which would lose the picked link's fill too. */
.lui-calendar-picked, a.lui-calendar-picked:hover { background: var(--lui-primary); color: var(--lui-on-primary); }
.lui-calendar-day:has(.lui-calendar-radio:checked) { background: var(--lui-primary); color: var(--lui-on-primary); }
.lui-calendar-off { color: var(--lui-muted); opacity: 0.5; cursor: not-allowed; }
.lui-calendar-day .lui-calendar-radio { position: absolute; inset: 0; opacity: 0; margin: 0; cursor: inherit; }
.lui-calendar-day:has(.lui-calendar-radio:focus-visible) { outline: 3px solid color-mix(in srgb, var(--lui-ring) 50%, transparent); }
.lui-calendar-dots { position: absolute; bottom: 3px; display: flex; gap: 2px; }
.lui-calendar-dots span { width: 4px; height: 4px; border-radius: 50%; background: currentColor; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip_through_day_numbers() {
        for s in [
            "1970-01-01",
            "2000-02-29",
            "2026-09-24",
            "2100-12-31",
            "1600-03-01",
        ] {
            let d = Date::parse(s).unwrap();
            assert_eq!(Date::from_days(d.days()), d);
            assert_eq!(d.to_string(), s);
        }
        assert_eq!(
            Date::parse("2026-09-24").unwrap().weekday(),
            3,
            "a Thursday"
        );
        assert_eq!(Date::parse("2026-02-29"), None);
        assert_eq!(
            Date::parse("2026-12-15").unwrap().add_months(1).to_string(),
            "2027-01-01"
        );
        assert_eq!(
            Date::parse("2026-01-31")
                .unwrap()
                .add_months(-1)
                .to_string(),
            "2025-12-01"
        );
        assert_eq!(
            Date::parse("2026-03-01").unwrap().add_days(-1).to_string(),
            "2026-02-28"
        );
    }

    #[test]
    fn the_grid_covers_the_month_in_whole_weeks() {
        let ui = Ui::from_request("/c", "month.d=2026-02", "");
        let m = ui.calendar("d").today("2026-02-10").render().into_string();
        // February 2026 starts on a Sunday: Monday-first needs the 26th..31st of January first.
        assert!(m.contains("2026-01-26") && !m.contains("2026-01-25"));
        assert_eq!(
            m.matches("<tr>").count(),
            1 + 5,
            "a header row and five weeks"
        );
        let sunday = ui
            .calendar("d")
            .today("2026-02-10")
            .sunday_first()
            .render()
            .into_string();
        assert!(sunday.contains(r#"abbr="Sunday">Su</th><th scope="col" abbr="Monday">"#));
        assert_eq!(
            sunday.matches("<tr>").count(),
            1 + 4,
            "Sunday-first February 2026 is four weeks"
        );
    }

    #[test]
    fn bounds_hide_month_links_and_block_days() {
        let ui = Ui::from_request("/c", "month.d=2026-09&d=2026-09-05", "");
        let m = ui
            .calendar("d")
            .today("2026-09-24")
            .min("2026-09-10")
            .max("2026-09-30")
            .render()
            .into_string();
        assert!(m.matches(r#"aria-disabled="true""#).count() >= 2);
        assert!(
            !m.contains("lui-calendar-picked"),
            "a picked day out of range is ignored"
        );
        assert!(
            m.contains(r#"<span class="lui-calendar-day lui-calendar-off" aria-disabled="true">"#)
        );
    }
}
