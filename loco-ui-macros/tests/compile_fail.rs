//! The errors a misspelled attribute, a missing required argument and a wrong value type give,
//! pinned so each keeps pointing at what the caller wrote.

#[test]
fn errors_point_at_what_was_written() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}
