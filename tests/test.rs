use anyhow::Result;
use wit_parser::Interface;

fn parse_str(s: &str) -> Result<String> {
    witgen_macro_helper::parse_str(s)
}

fn parse_wit_str(s: &str) -> Result<Interface> {
    Interface::parse("a", s)
}

fn parse(s: &str) {
    let res = parse_str(s).expect(s);
    parse_wit_str(&res).expect(&res);
}

#[test]
fn test() {
    let t = trybuild::TestCases::new();
    std::env::set_var("WITGEN_ENABLED", "true");
    t.pass("tests/files/success.rs");
}

#[test]
fn enum_simple() {
    let simple = "
enum MyEnum {
    Unit,
    TupleVariant(String, i32),
}
";
    parse(simple);
}

#[test]
fn enum_complicated() {
    let simple = "
enum MyEnum {
    Unit,
    TupleVariant(String, i32),
    HasNames { arg_one: u32, arg_two: String},
    HasMoreNames { arg_one: u32, arg_two: String, arg_three: (String,), arg_bool: bool},
}
";
    parse(simple);
}
