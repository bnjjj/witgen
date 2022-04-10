use std::str::FromStr;
use std::{fs::read, path::PathBuf};

use anyhow::Result;
use cargo_witgen::Witgen;
use difference::assert_diff;
use wit_parser::Interface;
use witgen_macro_helper::Wit;

fn parse_str(s: &str) -> Result<String> {
    Wit::from_str(s).map(|wit| wit.to_string())
}

fn parse_wit_str(s: &str) -> Result<Interface> {
    Interface::parse("a", s)
}

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
fn test_diff() {
    let witgen = Witgen {
        input_dir: PathBuf::from(&"examples/my_witgen_example"),
        output: PathBuf::from(&"index.wit"),
        prefix_file: vec![],
        prefix_string: vec![],
        stdout: false,
        input: None,
    };
    let wit = witgen.generate_str().unwrap();
    let path = &PathBuf::from(&"examples/my_witgen_example/index.wit");
    let file_str = String::from_utf8(read(path).unwrap()).unwrap();
    assert_diff!(&file_str, &wit, "", 0);
}
