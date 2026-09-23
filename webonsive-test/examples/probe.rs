//! Render one HTML file through Blitz and print the box of every selector given.
//!
//! `cargo run -p webonsive-test --example probe -- page.html out.png "table td" ".x"`

use webonsive_test::Page;

fn main() {
    let mut args = std::env::args().skip(1);
    let html = std::fs::read_to_string(args.next().expect("html file")).unwrap();
    let out = args.next().expect("output png");
    let mut page = Page::from_html(html);
    for selector in args {
        println!("{selector:24} display={:?} bbox={:?}", page.display(&selector), page.bbox(&selector));
    }
    page.screenshot(&out).unwrap();
    println!("wrote {out}");
}
