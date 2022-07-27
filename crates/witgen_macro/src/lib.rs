#![deny(warnings)]
use proc_macro::TokenStream;
use quote::quote;
use syn::ItemImpl;
use witgen_macro_helper::visitor::ImplVisitor;

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
    if let Ok(mut input) = syn::parse::<ItemImpl>(item.clone()) {
        // This converts attributes paths, e.g. #[path_macro], into a doc string, e.g. ///@path_macro
        ImplVisitor::path_attrs_to_docs(&mut input);
        quote! {#input}.into()
    } else {
        item
    }
}
