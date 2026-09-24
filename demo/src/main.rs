//! `cargo run -p demo` serves the component demo on http://127.0.0.1:3000 (`PORT` overrides
//! the port; the checks use 3001 so they never kill a server you are looking at).
//! `cargo run -p demo -- spec` prints `spec/components.json`; `-- spec write` regenerates
//! that file and the README feature matrix from `axum_nojs::spec::SPECS`.

use std::path::Path;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["spec"] => print!("{}", axum_nojs::spec::to_json()),
        ["spec", "write"] => write_spec(),
        [] => serve().await,
        other => eprintln!("unknown arguments {other:?}; try `spec`, `spec write`, or nothing"),
    }
}

async fn serve() {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .unwrap();
    println!("http://127.0.0.1:{port}");
    axum::serve(listener, demo::router()).await.unwrap();
}

/// Regenerate `spec/components.json` and the README matrix between its markers.
fn write_spec() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    std::fs::write(
        root.join("spec/components.json"),
        axum_nojs::spec::to_json(),
    )
    .unwrap();
    let readme_path = root.join("README.md");
    let readme = std::fs::read_to_string(&readme_path).unwrap();
    let (start, end) = ("<!-- matrix:start -->", "<!-- matrix:end -->");
    let a = readme.find(start).expect("README matrix start marker") + start.len();
    let b = readme.find(end).expect("README matrix end marker");
    let updated = format!(
        "{}\n{}{}",
        &readme[..a],
        axum_nojs::spec::markdown_table(),
        &readme[b..]
    );
    std::fs::write(&readme_path, updated).unwrap();
    println!("wrote spec/components.json and README.md feature matrix");
}
