//! `lui!` expands to the builder chain a route would write by hand: one test per rule, each
//! comparing the two forms' HTML.

use loco_ui::prelude::*;

fn ui() -> Ui {
    Ui::from_request("/", "", "")
}

fn same(a: Markup, b: Markup) {
    assert_eq!(a.into_string(), b.into_string());
}

/// A capitalized name is `ui.<snake_case>(required..)`; `;` ends it; plain Maud around it is
/// untouched.
#[test]
fn a_component_is_its_ui_method() {
    let ui = ui();
    same(
        lui! {
            h1.title { "Empty" }
            EmptyState("Nothing yet");
            DatePicker("due", "Due");
        },
        html! {
            h1.title { "Empty" }
            (ui.empty_state("Nothing yet"))
            (ui.date_picker("due", "Due"))
        },
    );
}

/// No required arguments: no parentheses. A block that is not items is the body.
#[test]
fn a_block_of_markup_is_the_body() {
    let ui = ui();
    same(
        lui! {
            Card title="Plan" { p { "Pro" } }
            Dialog("Delete account") small danger confirm=("Delete", "/delete") {
                p { "This cannot be undone." }
                Input("reason", "Why") placeholder="Moving on";
            }
        },
        html! {
            (ui.card().title("Plan").body(html! { p { "Pro" } }))
            (ui.dialog("Delete account").small().danger().confirm("Delete", "/delete").body(html! {
                p { "This cannot be undone." }
                (ui.input("reason", "Why").placeholder("Moving on"))
            }))
        },
    );
}

/// `x="v"`, `x=(expr)`, `x=-1`, bare `x`, `x[cond]`, `x=[option]` and `x=|i| { .. }`.
#[test]
fn attributes_are_setters() {
    let ui = ui();
    let (yes, no) = (true, false);
    let (some, none): (Option<&str>, Option<&str>) = (Some("Why?"), None);
    let max = 40;
    same(
        lui! {
            Input("a", "A") maxlength=(max) required autofocus[yes] search[no] help=[some] error=[none];
            Counter("/count", 3) min=-5 step=(2);
            Pager("/p", 3) per_page=2 rows=|i| { p { "Row " (i) } }
        },
        html! {
            (ui.input("a", "A").maxlength(max).required().autofocus().help("Why?"))
            (ui.counter("/count", 3).min(-5).step(2))
            (ui.pager("/p", 3).per_page(2).rows(|i| html! { p { "Row " (i) } }))
        },
    );
}

/// Items: arguments, modifiers as their attributes, a block as the last argument, `||` for a
/// closure, `()` for none, and a block of items continuing the chain.
#[test]
fn items_are_adders_with_their_modifiers() {
    let ui = ui();
    let title = "Use";
    same(
        lui! {
            Tabs("demo") select_below {
                tab "Install" { p { "cargo add" } }
                tab (title) badge=3 { p { "Call it." } }
                lazy "Why" || { p { "Later." } }
            }
            Form("/save") {
                text "name" "Name" required maxlength=40;
                email "email" "Email";
            }
            Menu("More") {
                link "Docs" "/docs";
                separator();
                action "Delete" "/delete" danger;
            }
            Kanban("/move") {
                column "todo" "To do" limit=3 {
                    card "a" "Write" note="Draft first";
                    card "b" "Test";
                }
                column "done" "Done";
            }
        },
        html! {
            (ui.tabs("demo").select_below()
                .tab("Install", html! { p { "cargo add" } })
                .tab(title, html! { p { "Call it." } }).badge(3)
                .lazy("Why", || html! { p { "Later." } }))
            (ui.form("/save").text("name", "Name").required().maxlength(40).email("email", "Email"))
            (ui.menu("More").link("Docs", "/docs").separator().action("Delete", "/delete").danger())
            (ui.kanban("/move")
                .column("todo", "To do").limit(3)
                .card("a", "Write").note("Draft first")
                .card("b", "Test")
                .column("done", "Done"))
        },
    );
}

/// `@for`, `@if` / `@else if` / `@else`, `@match` and `@let` among items.
#[test]
fn control_flow_among_items() {
    let ui = ui();
    let projects = [("Site", 3), ("App", 0), ("Docs", 1)];
    let status: Option<&str> = Some("beta");
    let by_hand = {
        let mut tabs = ui.tabs("p");
        for (name, open) in projects {
            tabs = tabs.tab(name, html! { (open) });
            if open > 2 {
                tabs = tabs.badge(open);
            } else if open == 0 {
                tabs = tabs.badge(0);
            }
        }
        tabs = tabs.tab("Status", html! { "beta" });
        tabs.tab("Total", html! { "4" })
    };
    same(
        lui! {
            Tabs("p") {
                @for (name, open) in projects {
                    @if open > 2 {
                        tab (name) badge=(open) { (open) }
                    } @else if open == 0 {
                        tab (name) badge=0 { (open) }
                    } @else {
                        tab (name) { (open) }
                    }
                }
                @match status {
                    Some(s) => { tab "Status" { (s) } }
                    None => tab "Status" { "none" },
                }
                @let total: usize = projects.iter().map(|p| p.1).sum();
                tab "Total" { (total) }
            }
        },
        html! { (by_hand) },
    );
}

/// Plain Maud passes through, `Some(..)` in a markup `@match` stays a pattern, brace attribute
/// values stay values, and components nest inside elements and item bodies.
#[test]
fn plain_maud_passes_through() {
    let ui = ui();
    let who: Option<&str> = ["Ada"].first().copied();
    let on = true;
    same(
        lui! {
            @match who {
                Some(name) => p { "Hi " (name) },
                None => p { "Hi" },
            }
            a href={ "/u/" (who.unwrap_or("-")) } .active[on] { "Profile" }
            @if on { section { Badge("New"); } }
            Tabs("n") { tab "One" { Badge("Inside") secondary; } }
        },
        html! {
            @match who {
                Some(name) => p { "Hi " (name) },
                None => p { "Hi" },
            }
            a href={ "/u/" (who.unwrap_or("-")) } .active[on] { "Profile" }
            @if on { section { (ui.badge("New")) } }
            (ui.tabs("n").tab("One", html! { (ui.badge("Inside").secondary()) }))
        },
    );
}

/// `lui!(ctx => ..)` names the `Ui` when it is not called `ui`.
#[test]
fn another_name_for_ui() {
    let ctx = ui();
    same(
        lui!(ctx => Badge("Hi") danger;),
        html! { (ctx.badge("Hi").danger()) },
    );
}

/// README's first example, in `lui!` and by hand, renders the same.
#[test]
fn the_readme_example() {
    let ui = Ui::from_request("/account", "", "lui-flash=Saved.");
    let name = String::from("Ada");
    same(
        lui! {
            Flash;
            Form("/account") submit="Save" { text "name" "Name" required value=(&name); }
            Dialog("Delete account") danger confirm=("Delete", "/account/delete") {
                p { "This cannot be undone." }
            }
        },
        html! {
            (ui.flash())
            (ui.form("/account").text("name", "Name").required().value(&name).submit("Save"))
            (ui.dialog("Delete account").danger().confirm("Delete", "/account/delete")
                .body(html! { p { "This cannot be undone." } }))
        },
    );
}
