#![deny(warnings)]
use proc_macro::TokenStream;

// use witgen_macro_helper::parse_and_write_to_file;

/// Proc macro attribute to help cargo-witgen to generate right definitions in `.wit` file
/// ```no_run
/// use witgen::witgen;
///
/// #[witgen]
/// struct TestStruct {
///     inner: String,
/// }
///
/// #[witgen]
/// enum TestEnum {
///     Unit,
///     Number(u64),
///     String(String),
/// }
///
/// #[witgen]
/// fn test(other: Vec<u8>, test_struct: TestStruct, other_enum: TestEnum) -> Result<(String, i64), String> {
///     Ok((String::from("test"), 0i64))
/// }
/// ```
#[proc_macro_attribute]
pub fn witgen(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
