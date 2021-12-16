#![allow(dead_code, unused_variables)]
use witgen::witgen;

#[witgen]
enum MyEnum {
    Unit,
    Tuple(String, i32),
}

#[witgen]
fn test_simple(array: Vec<u8>) -> String {
    String::from("test")
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
struct TestBis {
    coucou: String,
    btes: Vec<u8>,
}

#[witgen]
struct TestTuple(usize, String);

#[witgen]
struct TestStruct {
    inner: String,
}

#[witgen]
enum TestEnum {
    Unit,
    Number(u64),
    String(String),
}

#[witgen]
fn test_tuple(other: Vec<u8>, test_struct: TestStruct, other_enum: TestEnum) -> (String, i64) {
    (String::from("test"), 0i64)
}
