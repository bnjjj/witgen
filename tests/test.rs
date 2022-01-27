
#[test]
fn test() {
    let t = trybuild::TestCases::new();
    std::env::set_var("WITGEN_ENABLED", "true");
    t.pass("tests/files/success.rs");
}