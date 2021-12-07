# witgen

witgen is a library to help you generate wit definitions in a wit file for WebAssembly

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

```
$ cargo witgen generate
```

- It will generate a `witgen.wit` file at the root of your package:

```
test : function(other: list <u8>, number: u8, othernum: s32) -> (string, s64)
```

## Roadmap:

- Implement proc macro `#[witgen]` to put on enum, struct and functions
- Add proc_macro options (rename, file ?, ...)
