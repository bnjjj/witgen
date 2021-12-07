# witgen

![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
![Rust](https://github.com/bnjjj/witgen/workflows/Rust/badge.svg)
[![Version](https://img.shields.io/crates/v/witgen.svg)](https://crates.io/crates/witgen)
[![Docs.rs](https://docs.rs/witgen/badge.svg)](https://docs.rs/witgen)

witgen is a library to help you generate [wit definitions](https://github.com/bytecodealliance/wit-bindgen/blob/main/WIT.md) in a wit file for WebAssembly. Using this lib in addition to [wit-bindgen](https://github.com/bytecodealliance/wit-bindgen) will help you to import/export types and functions from/to wasm module.

## Getting started

- Put this dependency in your `Cargo.toml`

```toml
witgen = "0.2"
```

- Install `cargo witgen` CLI

```bash
$ cargo install cargo-witgen
```

## Examples

- Into your Rust code:

```rust
use witgen::witgen;

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
fn test(other: Vec<u8>, test_struct: TestStruct, other_enum: TestEnum) -> Result<(String, i64), String> {
    Ok((String::from("test"), 0i64))
}
```

- Then you can launch (at the root of your package):

```bash
$ cargo witgen generate
```

- It will generate a `witgen.wit` file at the root of your package:

```wit
record TestStruct {
    inner: string
}

variant TestEnum {
    Unit,
	Number(u64),
	String(string),
}

test : function(other: list <u8>, test_struct: TestStruct, other_enum: TestEnum) -> expected<tuple<string, s64>>
```

## Development

It's a very minimal version, it doesn't already support all kinds of types but the main used are supported. I made it to easily generate `.wit` files for my need. Feel free to create issues or pull-requests if you need something. I will be happy to help you !
