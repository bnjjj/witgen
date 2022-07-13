use std::str::FromStr;
use std::{fs::read, path::PathBuf};

use anyhow::Result;
use cargo_witgen::Witgen;
use wit_parser::Interface;
use k9::{assert_matches_snapshot};
use witgen_macro_helper::{DefaultResolver, Wit, Resolver};

// struct Empty;

// impl Resolve for Empty {

// }

fn parse_str(s: &str) -> Result<String> {
    Wit::from_str(s).map(|wit| wit.to_string())
}

fn parse_wit_str(s: &str) -> Result<Interface> {
    DefaultResolver::parse_wit_interface_default("a", s)
}

// fn parse_wit_str_with_path(s: &str) -> Result<Interface> {
//   Interface::parse_with("a", s, |path| resolve_wit_ )
// }

fn parse(s: &str) {
    let res = parse_str(s).expect(s);
    parse_wit_str(&res).expect(&res);
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

#[test]
fn test_diff() -> Result<()> {
    let witgen = Witgen {
        input_dir: PathBuf::from(&"examples/my_witgen_example"),
        output: PathBuf::from(&"index.wit"),
        prefix_file: vec![],
        prefix_string: vec![],
        stdout: false,
        input: None,
        cargo: Default::default(),
        skip_resolve: false,
    };

    let wit = witgen.generate_str(witgen.read_input()?)?;
    let wit = witgen.resolve(&wit)?;
    assert_matches_snapshot!(wit);
    // assert_equal!(file_str, wit);
    Ok(())
}

#[test]
fn floats() {
    let simple = r#"
#[witgen]
type Float32Bit = f32;

#[witgen]
type Float64Bit = f64;
"#;
    println!("{:?}", parse_wit_str(&parse_str(simple).unwrap()).unwrap())
}
