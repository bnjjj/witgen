#![allow(dead_code, unused_variables)]
use std::collections::HashMap;

use witgen::witgen;

#[witgen]
enum MyEnum {
    Unit,
    TupleVariant(String, i32),
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
    /// Doc comment over Unit variant in struct
    Unit,
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