#![deny(warnings)]
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{parse, ItemEnum, ItemFn, ItemStruct, ItemType};
use witgen_macro_helper::{
    gen_wit_enum, gen_wit_function, gen_wit_struct, gen_wit_type_alias, handle_error,
};

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
    let strukt = parse::<ItemStruct>(item.clone());
    if let Ok(strukt) = &strukt {
        handle_error!(gen_wit_struct(strukt));
        return item;
    }

    let func = parse::<ItemFn>(item.clone());
    if let Ok(func) = &func {
        handle_error!(gen_wit_function(func));
        return item;
    }

    let enm = parse::<ItemEnum>(item.clone());
    if let Ok(enm) = &enm {
        handle_error!(gen_wit_enum(enm));
        return item;
    }

    let type_alias = parse::<ItemType>(item.clone());
    if let Ok(type_alias) = &type_alias {
        handle_error!(gen_wit_type_alias(type_alias));
        return item;
    }

    syn::Error::new_spanned(
        proc_macro2::TokenStream::from(item),
        "Cannot put wit_generator proc macro on this kind of item",
    )
    .to_compile_error()
    .into()
}
