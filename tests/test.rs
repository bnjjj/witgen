use std::{path::PathBuf, fs::read};
use std::str::FromStr;

use anyhow::Result;
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
  let wit = witgen_macro_helper::parse_crate_as_file(&PathBuf::from(&"examples/my_witgen_example/src/lib.rs")).unwrap();
  let path = &PathBuf::from(&"examples/my_witgen_example/index.wit");
  let file_str = String::from_utf8(read(path).unwrap()).unwrap();
  let (_, rest) = file_str.split_once("\n").unwrap();
  assert_diff!(rest.trim_matches('\n'), &wit.to_string(), "", 1); 

}