//! `cargo run -p demo` serves the component demo on http://127.0.0.1:3000.
//! `cargo run -p demo -- spec` prints `spec/components.json`; `-- spec write` regenerates
//! that file and the README feature matrix from `webonsive::spec::SPECS`.

use std::path::Path;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["spec"] => print!("{}", webonsive::spec::to_json()),
        ["spec", "write"] => write_spec(),
        [] => serve().await,
        other => eprintln!("unknown arguments {other:?}; try `spec`, `spec write`, or nothing"),
    }
}

async fn serve() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("http://127.0.0.1:3000");
    axum::serve(listener, demo::router()).await.unwrap();
}

/// Regenerate `spec/components.json` and the README matrix between its markers.
fn write_spec() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    std::fs::write(root.join("spec/components.json"), webonsive::spec::to_json()).unwrap();
    let readme_path = root.join("README.md");
    let readme = std::fs::read_to_string(&readme_path).unwrap();
    let (start, end) = ("<!-- matrix:start -->", "<!-- matrix:end -->");
    let a = readme.find(start).expect("README matrix start marker") + start.len();
    let b = readme.find(end).expect("README matrix end marker");
    let updated = format!("{}\n{}{}", &readme[..a], webonsive::spec::markdown_table(), &readme[b..]);
    std::fs::write(&readme_path, updated).unwrap();
    println!("wrote spec/components.json and README.md feature matrix");
}
