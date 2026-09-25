//! `cargo lui install`: set up a Loco app for loco-ui in one command. `cargo lui auth`: add
//! the account pages.
//!
//! ```sh
//! cargo install --git https://github.com/luis-serrano-l/loco-ui loco-ui --bin cargo-lui
//! cargo lui install                  # in the app's directory
//! cargo lui install path/to/app      # or name it
//! cargo lui auth                     # then the account pages
//! # from a loco-ui checkout, without installing:
//! cargo run -p loco-ui -- install path/to/app --dep-path "$PWD/loco-ui"
//! ```
//!
//! It writes, and prints one line per step:
//! - `.loco-templates/scaffold/api/{controller,dto}.t`, the scaffold that makes
//!   `cargo loco generate scaffold` write Maud views and HTML controllers;
//! - the initializer line in `src/app.rs` (`Box::new(loco_ui::loco::Initializer)`);
//! - `src/views/layout.rs`, a page with a header, and `pub mod layout;` in `src/views/mod.rs`;
//! - `loco-ui` (git, or `--dep-path`) and `maud` under `[dependencies]` in `Cargo.toml`.
//!
//! `auth` writes the account pages (sign in, sign up, sign out, forgot and reset password,
//! email verification, magic link) as no-script forms on the starter's `users` model and
//! `AuthMailer` (`loco new` with a database):
//! - `src/controllers/account.rs` and `src/views/account.rs` (from `loco-templates/auth/`),
//!   their `pub mod account;` lines, and `.add_route(controllers::account::routes())` in
//!   `src/app.rs`;
//! - the links in the starter's mails (`src/mailers/auth/*/{html,text}.t`) pointed at those
//!   pages (`/verify/<token>`, `/reset/<token>`, `/magic-link/<token>`);
//! - `location: { from: Cookie, name: auth }` under `auth.jwt` in each `config/*.yaml`, so
//!   Loco's `auth::JWT` reads the token from the `HttpOnly` cookie the pages set.
//!
//! Both are idempotent: a second run changes nothing. A file that exists with other content is left
//! alone and reported (`--force` overwrites the templates and the layout). Standard library
//! only, so it builds without the crate's features.

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

const CONTROLLER: &str = include_str!("../../loco-templates/scaffold/api/controller.t");
const DTO: &str = include_str!("../../loco-templates/scaffold/api/dto.t");
const ACCOUNT_CONTROLLER: &str = include_str!("../../loco-templates/auth/controller.rs");
const ACCOUNT_VIEWS: &str = include_str!("../../loco-templates/auth/views.rs");
const ACCOUNT_ROUTE: &str = ".add_route(controllers::account::routes())";
/// The starter's mail links (to its JSON API and SPA) and where the account pages answer.
const MAIL_LINKS: [(&str, &str); 3] = [
    ("{{host}}/api/auth/verify/", "{{host}}/verify/"),
    ("{{host}}/reset#", "{{host}}/reset/"),
    ("{{host}}/api/auth/magic-link/", "{{host}}/magic-link/"),
];
const JWT_LOCATION: &str = "    # loco-ui's account pages keep the token in this cookie.
    location:
      from: Cookie
      name: auth
";
const INITIALIZER: &str = "Box::new(loco_ui::loco::Initializer)";
const GIT: &str = "https://github.com/luis-serrano-l/loco-ui";

const LAYOUT: &str = r#"//! The page every view renders into: a header with a home link (add yours), then the view.
//! Written by `cargo lui install`; edit freely.

use loco_ui::prelude::*;

/// `body` inside the app's header, as a full page titled `title`.
pub fn page(ui: &Ui, title: &str, body: Markup) -> Page {
    ui.page(
        title,
        html! {
            header {
                (ui.cluster().body(html! {
                    a href="/" { strong { "Home" } }
                }))
            }
            main { (body) }
        },
    )
}
"#;

const USAGE: &str = "usage: cargo lui install [APP_DIR] [--dep-path PATH] [--force]
       cargo lui auth [APP_DIR] [--force]";

fn main() -> ExitCode {
    // `cargo lui ..` runs this binary as `cargo-lui lui ..`.
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("lui") {
        args.remove(0);
    }
    let command: fn(&Options) -> Result<(), String> = match args.first().map(String::as_str) {
        Some("install") => install,
        Some("auth") => auth,
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let mut opts = Options {
        app: PathBuf::from("."),
        dep_path: None,
        force: false,
    };
    let mut rest = args.into_iter().skip(1);
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--force" => opts.force = true,
            "--dep-path" => match rest.next() {
                Some(p) => opts.dep_path = Some(p),
                None => {
                    eprintln!("{USAGE}");
                    return ExitCode::from(2);
                }
            },
            s if s.starts_with('-') => {
                eprintln!("unknown option {s}\n{USAGE}");
                return ExitCode::from(2);
            }
            _ => opts.app = PathBuf::from(arg),
        }
    }
    match command(&opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

struct Options {
    app: PathBuf,
    dep_path: Option<String>,
    force: bool,
}

fn loco_app(app: &Path) -> Result<(), String> {
    if !app.join("src/app.rs").is_file() || !app.join("Cargo.toml").is_file() {
        return Err(format!(
            "{} is not a Loco app (no Cargo.toml and src/app.rs)",
            app.display()
        ));
    }
    Ok(())
}

fn install(o: &Options) -> Result<(), String> {
    let app = &o.app;
    loco_app(app)?;
    let templates = app.join(".loco-templates/scaffold/api");
    write_file(&templates.join("controller.t"), CONTROLLER, o.force)?;
    write_file(&templates.join("dto.t"), DTO, o.force)?;
    write_file(&app.join("src/views/layout.rs"), LAYOUT, o.force)?;
    edit(&app.join("src/views/mod.rs"), |s| {
        add_line(s, "pub mod layout;")
    })?;
    edit(&app.join("src/app.rs"), add_initializer)?;
    let dep = match &o.dep_path {
        Some(p) => format!(r#"loco-ui = {{ path = "{p}", features = ["loco"] }}"#),
        None => format!(r#"loco-ui = {{ git = "{GIT}", features = ["loco"] }}"#),
    };
    edit(&app.join("Cargo.toml"), |s| {
        let s = add_dependency(s, "loco-ui", &dep)?;
        add_dependency(&s, "maud", r#"maud = "0.27""#)
    })?;
    println!("done: `cargo loco generate scaffold <model> <fields..>` now writes Maud views");
    Ok(())
}

fn auth(o: &Options) -> Result<(), String> {
    let app = &o.app;
    loco_app(app)?;
    let manifest = fs::read_to_string(app.join("Cargo.toml")).unwrap_or_default();
    if !manifest.contains("loco-ui") {
        return Err("no loco-ui dependency: run `cargo lui install` first".into());
    }
    let users = fs::read_to_string(app.join("src/models/users.rs")).unwrap_or_default();
    if !users.contains("RegisterParams") || !app.join("src/mailers/auth.rs").is_file() {
        return Err(
            "needs the starter's users model and AuthMailer (`loco new` with a database)".into(),
        );
    }
    write_file(
        &app.join("src/controllers/account.rs"),
        ACCOUNT_CONTROLLER,
        o.force,
    )?;
    write_file(&app.join("src/views/account.rs"), ACCOUNT_VIEWS, o.force)?;
    edit(&app.join("src/controllers/mod.rs"), |s| {
        add_line(s, "pub mod account;")
    })?;
    edit(&app.join("src/views/mod.rs"), |s| {
        add_line(s, "pub mod account;")
    })?;
    edit(&app.join("src/app.rs"), add_account_route)?;
    for mail in ["welcome", "forgot", "magic_link"] {
        for part in ["html.t", "text.t"] {
            let path = app.join("src/mailers/auth").join(mail).join(part);
            if path.is_file() {
                edit(&path, |s| Ok(point_mail_links(s)))?;
            }
        }
    }
    let config = app.join("config");
    let mut files: Vec<PathBuf> = fs::read_dir(&config)
        .map_err(|e| format!("{}: {e}", config.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "yaml"))
        .collect();
    files.sort();
    for path in files {
        edit(&path, add_jwt_location)?;
    }
    println!("done: /signin, /signup, /forgot and /magic-link answer with pages");
    Ok(())
}

/// Register the account routes right after `AppRoutes::..` in `fn routes`.
fn add_account_route(s: &str) -> Result<String, String> {
    if s.contains(ACCOUNT_ROUTE) {
        return Ok(s.to_string());
    }
    let missing = || format!("no `AppRoutes::` in `fn routes`; add `{ACCOUNT_ROUTE}` by hand");
    let at = s.find("fn routes").ok_or_else(missing)?;
    let at = at + s[at..].find("AppRoutes::").ok_or_else(missing)?;
    let end = at + s[at..].find('\n').ok_or_else(missing)? + 1;
    Ok(format!(
        "{}            {ACCOUNT_ROUTE}\n{}",
        &s[..end],
        &s[end..]
    ))
}

/// Point the starter's mail links at the account pages.
fn point_mail_links(s: &str) -> String {
    MAIL_LINKS
        .iter()
        .fold(s.to_string(), |s, (from, to)| s.replace(from, to))
}

/// Add the cookie location under `auth:` → `jwt:` unless that block names a location.
fn add_jwt_location(s: &str) -> Result<String, String> {
    let lines: Vec<&str> = s.split_inclusive('\n').collect();
    let Some(auth) = lines.iter().position(|l| l.trim_end() == "auth:") else {
        return Ok(s.to_string());
    };
    // The block runs to the next line that starts in the first column (not a comment).
    let end = lines[auth + 1..]
        .iter()
        .position(|l| l.starts_with(|c: char| !c.is_whitespace() && c != '#'))
        .map_or(lines.len(), |i| auth + 1 + i);
    let block = &lines[auth..end];
    if block
        .iter()
        .any(|l| l.trim_start().starts_with("location:"))
    {
        return Ok(s.to_string());
    }
    let Some(jwt) = block.iter().position(|l| l.trim_end() == "  jwt:") else {
        return Ok(s.to_string());
    };
    let at = auth + jwt + 1;
    Ok(format!(
        "{}{JWT_LOCATION}{}",
        lines[..at].concat(),
        lines[at..].concat()
    ))
}

/// Write `content` unless the file already holds it; another content is kept unless `force`.
fn write_file(path: &Path, content: &str, force: bool) -> Result<(), String> {
    match fs::read_to_string(path) {
        Ok(old) if old == content => println!("unchanged {}", path.display()),
        Ok(_) if !force => println!("kept      {} (edited; --force overwrites)", path.display()),
        _ => {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            }
            fs::write(path, content).map_err(|e| format!("{}: {e}", path.display()))?;
            println!("wrote     {}", path.display());
        }
    }
    Ok(())
}

/// Apply `f` to the file (created empty when missing) and write it back only if it changed.
fn edit(path: &Path, f: impl Fn(&str) -> Result<String, String>) -> Result<(), String> {
    let old = fs::read_to_string(path).unwrap_or_default();
    let new = f(&old).map_err(|e| format!("{}: {e}", path.display()))?;
    if new == old {
        println!("unchanged {}", path.display());
    } else {
        fs::write(path, new).map_err(|e| format!("{}: {e}", path.display()))?;
        println!("edited    {}", path.display());
    }
    Ok(())
}

/// Append `line` unless a line reads exactly that.
fn add_line(s: &str, line: &str) -> Result<String, String> {
    if s.lines().any(|l| l.trim() == line) {
        return Ok(s.to_string());
    }
    let mut out = s.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(line);
    out.push('\n');
    Ok(out)
}

/// Put the initializer first in the `vec![..]` that `fn initializers` returns.
fn add_initializer(s: &str) -> Result<String, String> {
    if s.contains(INITIALIZER) {
        return Ok(s.to_string());
    }
    let missing = || format!("no `vec![` in `fn initializers`; add `{INITIALIZER}` to it by hand");
    let at = s.find("fn initializers").ok_or_else(missing)?;
    let open = at + s[at..].find("vec![").ok_or_else(missing)? + "vec![".len();
    let empty = s[open..].trim_start().starts_with(']');
    let insert = if empty {
        INITIALIZER.to_string()
    } else {
        format!("{INITIALIZER}, ")
    };
    Ok(format!("{}{insert}{}", &s[..open], &s[open..]))
}

/// Add `line` as the first entry of `[dependencies]` unless `name` is already a dependency.
fn add_dependency(s: &str, name: &str, line: &str) -> Result<String, String> {
    let mut section = "";
    for l in s.lines() {
        let t = l.trim();
        if t.starts_with('[') {
            section = t;
        } else if section == "[dependencies]" && t.split(['=', ' ', '.']).next() == Some(name) {
            return Ok(s.to_string());
        }
    }
    let at = s
        .find("[dependencies]\n")
        .ok_or("no [dependencies] section")?;
    let at = at + "[dependencies]\n".len();
    Ok(format!("{}{line}\n{}", &s[..at], &s[at..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_initializer_goes_first_once() {
        let fresh = "async fn initializers(_ctx: &AppContext) -> Result<..> {\n    Ok(vec![])\n}";
        let once = add_initializer(fresh).unwrap();
        assert!(once.contains("Ok(vec![Box::new(loco_ui::loco::Initializer)])"));
        assert_eq!(add_initializer(&once).unwrap(), once);
        let other = "fn initializers() { Ok(vec![Box::new(Mine)]) }";
        let added = add_initializer(other).unwrap();
        assert!(added.contains("vec![Box::new(loco_ui::loco::Initializer), Box::new(Mine)]"));
        assert!(add_initializer("fn routes() {}").is_err());
    }

    #[test]
    fn a_dependency_is_added_once_and_only_under_dependencies() {
        let toml = "[workspace.dependencies]\nmaud = \"1\"\n\n[dependencies]\nserde = \"1\"\n";
        let added = add_dependency(toml, "maud", "maud = \"0.27\"").unwrap();
        assert!(added.contains("[dependencies]\nmaud = \"0.27\"\nserde"));
        assert_eq!(
            add_dependency(&added, "maud", "maud = \"0.27\"").unwrap(),
            added
        );
        let dotted = "[dependencies]\nloco-ui.path = \"x\"\n";
        assert_eq!(add_dependency(dotted, "loco-ui", "..").unwrap(), dotted);
    }

    #[test]
    fn the_jwt_reads_the_cookie() {
        let yaml =
            "server:\n  port: 1\nauth:\n  # JWT\n  jwt:\n    secret: x\n\ndatabase:\n  uri: y\n";
        let added = add_jwt_location(yaml).unwrap();
        assert!(added.contains("  jwt:\n    # loco-ui's account pages keep the token in this cookie.\n    location:\n      from: Cookie\n      name: auth\n    secret: x"));
        assert_eq!(add_jwt_location(&added).unwrap(), added);
        assert_eq!(add_jwt_location("server: {}\n").unwrap(), "server: {}\n");
    }

    #[test]
    fn the_account_route_follows_app_routes() {
        let app = "fn routes(_ctx: &AppContext) -> AppRoutes {\n        AppRoutes::with_default_routes() // controller routes below\n            .add_route(controllers::auth::routes())\n    }";
        let added = add_account_route(app).unwrap();
        assert!(added.contains("below\n            .add_route(controllers::account::routes())\n            .add_route(controllers::auth"));
        assert_eq!(add_account_route(&added).unwrap(), added);
    }

    #[test]
    fn mail_links_point_at_the_pages() {
        let text = "{{host}}/api/auth/verify/{{verifyToken}} {{host}}/reset#{{resetToken}}";
        assert_eq!(
            point_mail_links(text),
            "{{host}}/verify/{{verifyToken}} {{host}}/reset/{{resetToken}}"
        );
    }

    #[test]
    fn a_module_line_is_appended_once() {
        let once = add_line("pub mod auth;", "pub mod layout;").unwrap();
        assert_eq!(once, "pub mod auth;\npub mod layout;\n");
        assert_eq!(add_line(&once, "pub mod layout;").unwrap(), once);
    }
}
