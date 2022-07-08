#![allow(dead_code, unused_variables)]
use std::collections::HashMap;
use witgen::witgen;

#[witgen]
use example_dep::ExternalDep;

mod extra_type;
use extra_type::*;




#[witgen]
enum Colors {
    Red,
    Green,
    Blue,
}

#[witgen]
enum MyEnum {
    UnitType,
    TupleVariant(String, i32),
}

#[witgen]
enum WithNamedFields {
    /// Example variant with named fields
    Example {
        /// Doc for inner string
        name: String,
    },
    UnitType,
    ATuple(String),
    /// Example of a big named field
    BigExample {
        /// Info about field
        field: u32,
        b: bool,
        s: String,
        a: Vec<u32>,
        a_tuple: (f64, HashMap<u32, MyEnum>),
    },
}

#[witgen]
fn test_simple(array: Vec<u8>) -> String {
    String::from("test")
}

#[witgen]
type NFTContractMetadata = String;

#[witgen]
pub struct InitArgs {
    owner_id: String,
    metadata: NFTContractMetadata,
}

#[witgen]
fn test_array(other: [u8; 32], number: u8, othernum: i32) -> (String, usize) {
    (String::from("test"), 0usize)
}

#[witgen]
fn test_vec(other: Vec<u8>, number: u8, othernum: i32) -> (String, usize) {
    (String::from("test"), 0usize)
}

#[witgen]
fn test_option(other: Vec<u8>, number: u8, othernum: i32) -> Option<(String, usize)> {
    Some((String::from("test"), 0usize))
}

#[witgen]
fn test_result(other: Vec<u8>, number: u8, othernum: i32) -> Result<(String, usize), String> {
    Ok((String::from("test"), 0usize))
}

#[witgen]
/// Here is a doc example to generate in wit file
struct TestBis {
    coucou: String,
    btes: Vec<u8>,
}

#[witgen]
/// Documentation over struct
/// in multi-line
struct TestTuple(usize, String);
#[witgen]
struct TestStruct {
    /// Doc comment over inner field in struct
    inner: String,
}

/// Documentation over enum
#[witgen]
enum TestEnum {
    /// Doc comment over UnitType variant in struct
    /// Two lines
    UnitType,
    Number(u64),
    /// Doc comment over String variant in struct
    StringVariant(String),
}

#[witgen]
fn test_tuple(other: Vec<u8>, test_struct: TestStruct, other_enum: TestEnum) -> (String, i64) {
    (String::from("test"), 0i64)
}

#[witgen]
struct HasHashMap {
    map: HashMap<String, TestStruct>,
}

#[witgen]
fn use_string_alias(s: StringAlias) -> StringAlias {
    s
}

fn has_no_macro() {}

#[witgen]
type Float32Bit = f32;

#[witgen]
type Float64Bit = f64;

#[witgen]
pub fn use_ext_dep() -> ExternalDep {
    String::from("hello")
}

/// This is an example wit interface
#[witgen]
trait ExampleInterface {
  fn foo() -> String;

  /// Has doc string
  /// With two lines
  fn f(w: WithNamedFields) -> TestEnum;
}
