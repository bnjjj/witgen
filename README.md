# witgen

![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
![Rust](https://github.com/bnjjj/witgen/workflows/Rust/badge.svg)
[![Version](https://img.shields.io/crates/v/witgen.svg)](https://crates.io/crates/witgen)
[![Docs.rs](https://docs.rs/witgen/badge.svg)](https://docs.rs/witgen)

> witgen is a library to help you generate wit definitions in a wit file for WebAssembly

# Getting started

- Put this dependency in your `Cargo.toml`

```toml
witgen = "0.1"
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
fn test(other: Vec<u8>, number: u8, othernum: i32) -> (String, i64) {
    (String::from("test"), 0i64)
}
```

- Then you can launch (at the root of your package):

```bash
$ cargo witgen generate
```

- It will generate a `witgen.wit` file at the root of your package:

```
test : function(other: list <u8>, number: u8, othernum: s32) -> (string, s64)
```

## Roadmap:

- Implement proc macro `#[witgen]` to put on enum, struct and functions
- Add proc_macro options (rename, file ?, ...)
