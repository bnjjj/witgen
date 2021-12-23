# witgen

![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
![Rust](https://github.com/bnjjj/witgen/workflows/Rust/badge.svg)
[![Version](https://img.shields.io/crates/v/witgen.svg)](https://crates.io/crates/witgen)
[![Docs.rs](https://docs.rs/witgen/badge.svg)](https://docs.rs/witgen)

witgen is a library and a CLI that helps you generate [wit definitions](https://github.com/bytecodealliance/wit-bindgen/blob/main/WIT.md) in a wit file for WebAssembly. Using this lib in addition to [wit-bindgen](https://github.com/bytecodealliance/wit-bindgen) will help you to import/export types and functions from/to wasm module.

## Getting started

_Goal:_ Generate a `.wit` file writing only Rust.

You will need both the library and the CLI.

### Preliminaries

- Create a new library project and move to it.

```bash
$ cargo new my_wit
$ cd my_wit
```

- Add `witgen` as a dependency in your `Cargo.toml`.

```bash
$ cargo add witgen
```

- Install `cargo witgen` CLI.

```bash
$ cargo install cargo-witgen
```

### Writing code

- Replace the content of your `lib.rs` by:

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
    StringVariant(String),
}

#[witgen]
fn test(other: Vec<u8>, test_struct: TestStruct, other_enum: TestEnum) -> Result<(String, i64), String> {
    // The following code is not part of the generated `.wit` file.
    // You may add an example implementation or just satisfy the compiler with a `todo!()`.
    Ok((String::from("test"), 0i64))
}
```

- Then you can launch the CLI (at the root of your package):

```bash
$ cargo witgen generate
```

- It will generate a `witgen.wit` file at the root of your package:

```wit
record test-struct {
    inner: string
}

variant test-enum {
    unit,
	number(u64),
	string-variant(string),
}

test : function(other: list <u8>, test-struct: test-struct, other-enum: test-enum) -> expected<tuple<string, s64>>
```

- You can find more complete examples [here](./examples)

## Limitations

For now using `#[witgen]` have some limitations:

- You can use the proc macro `#[witgen]` only on `struct`, `enum`, `type alias`, `function`
- Generic parameters or lifetime anotations are not supported
- Type `&str` is not supported (but you can use `String`)
- Named struct variants in `enum` are not already supported (examples `enum Test { NamedVariant: { inner: String } }` but this one is supported `enum Test { UnNamedVariant(String, usize) }`)
- References, `Box`, `Rc`, `Arc` and all types of smart pointers are not supported
- Methods are not supported
- There is no semantic analysis, which means if your `function`, `struct` or `enum` uses a non scalar type, you have to add `#[witgen]` where this type is declared (it won't fail at the compile time)

## Development

It's a very minimal version, it doesn't already support all kinds of types but the main ones are supported. I made it to easily generate `.wit` files for my need. Feel free to create issues or pull-requests if you need something. I will be happy to help you!
