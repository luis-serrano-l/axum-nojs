//! `cargo run -p demo` serves the component demo on http://127.0.0.1:3000.

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("http://127.0.0.1:3000");
    axum::serve(listener, demo::router()).await.unwrap();
}
