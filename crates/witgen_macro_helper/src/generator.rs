use std::fmt::Write;

use anyhow::{bail, Context, Result};
use heck::ToKebabCase;
use syn::{Attribute, Fields, ItemEnum, ItemFn, ItemStruct, ItemType, Lit, ReturnType, Type};

use crate::wit::{is_known_keyword, ToWitType};

/// Generate a wit record
/// ```rust
/// /// Document String
/// struct FooRecord {
///    a: string,
///    /// Comment field
///    b: Option<i32>,
/// }
/// ```
/// becomes
/// ```ts
/// /// Document String
/// record foo-record {
///   a: string,
///   b: option<s32>,
/// }
/// ```
///
pub fn gen_wit_struct(strukt: &ItemStruct) -> Result<String> {
    if !strukt.generics.params.is_empty() {
        bail!("doesn't support generic parameters with witgen");
    }

    let struct_name = strukt.ident.to_string().to_kebab_case();
    is_known_keyword(&struct_name)?;

    let mut is_tuple_struct = false;
    let attrs = strukt
        .fields
        .iter()
        .map(|field| {
            let field_name = match &field.ident {
                Some(ident) => ident.to_string().to_kebab_case() + ": ",
                None => {
                    is_tuple_struct = true;
                    String::new()
                }
            };
            is_known_keyword(&field_name)?;

            let comment = get_doc_comment(&field.attrs)?;

            let field_wit = format!("{}{}", field_name, field.ty.to_wit()?);
            match comment {
                Some(comment) => Ok(format!("{}    {}", comment, field_wit)),
                None => Ok(field_wit),
            }
        })
        .collect::<Result<Vec<String>>>()?;
    let attrs = if is_tuple_struct {
        attrs.join(", ")
    } else {
        attrs.join(",\n    ")
    };

    let content = if is_tuple_struct {
        format!("type {} = tuple<{}>\n", struct_name, attrs)
    } else {
        format!(
            r#"record {} {{
    {}
}}
"#,
            struct_name, attrs
        )
    };
    Ok(content)
}

/// Generate a wit enum
/// ```rust
/// /// Top comment
/// enum MyEnum {
///   /// comment case
///   Unit,
///   TupleVariant(String, i32)
/// }
/// ```
///
/// ```ts
/// /// Top comment
/// variant my-enum {
///   /// comment case
///   unit,
///   tuple-variant(tuple<string, s32>),
/// }
/// ```
pub fn gen_wit_enum(enm: &ItemEnum) -> Result<String> {
    if !enm.generics.params.is_empty() {
        bail!("doesn't support generic parameters with witgen");
    }

    let enm_name = enm.ident.to_string().to_kebab_case();
    is_known_keyword(&enm_name)?;

    let is_wit_enum = enm
        .variants
        .iter()
        .all(|v| matches!(v.fields, Fields::Unit));
    let mut named_types = String::new();
    let variants = enm
        .variants
        .iter()
        .map(|variant| {
            let ident = variant.ident.to_string().to_kebab_case();
            let comment = get_doc_comment(&variant.attrs)?;
            let variant_string = match &variant.fields {
                syn::Fields::Named(_named) => {
                    let fields = _named
                        .named
                        .iter()
                        .map(|field| {
                            field.ty.to_wit().map(|ty| {
                                let field_doc = get_doc_comment(&field.attrs)
                                    .unwrap_or(None)
                                    .as_ref()
                                    .map_or("".to_string(), |s| format!("    {}", s));

                                format!(
                                    "{}    {}: {}",
                                    field_doc,
                                    field.ident.as_ref().unwrap().to_string().to_kebab_case(),
                                    ty
                                )
                            })
                        })
                        .collect::<Result<Vec<String>>>()?
                        .join(",\n");
                    let inner_type_name = &format!("{}-{}", enm_name, ident);
                    let comment = comment.as_deref().unwrap_or_default();
                    named_types.push_str(&format!(
                        "{}record {} {{\n{}\n}}\n",
                        comment, inner_type_name, fields
                    ));
                    Ok(format!("{}({})", ident, inner_type_name))
                }
                syn::Fields::Unnamed(unamed) => {
                    let fields = unamed
                        .unnamed
                        .iter()
                        .map(|field| field.ty.to_wit())
                        .collect::<Result<Vec<String>>>()?
                        .join(", ");
                    is_known_keyword(&ident)?;

                    let formatted_field = if unamed.unnamed.len() > 1 {
                        format!("tuple<{}>", fields)
                    } else {
                        fields
                    };

                    Ok(format!("{}({})", ident, formatted_field))
                }
                syn::Fields::Unit => Ok(ident),
            };
            let comment = comment.map(|s| format!("    {}", s)).unwrap_or_default();
            variant_string.map(|v| format!("{}    {},", comment, v))
        })
        .collect::<Result<Vec<String>>>()?
        .join("\n");
    let ty = if is_wit_enum { "enum" } else { "variant" };
    let content = format!(
        r#"{} {} {{
{}
}}
"#,
        ty, enm_name, variants
    );

    Ok(format!("{}\n{}", content, named_types))
}

/// Generate a wit function
/// ```rust
/// /// Document String
/// fn foo(a: string, b: Option<i32>) -> Result<string> { Ok(a)}
/// ```
/// becomes
/// ```ts
/// /// Document String
/// foo: function(a: string, b: option<s32>) -> expected<string>
/// ```
///
pub fn gen_wit_function(func: &ItemFn) -> Result<String> {
    let signature = &func.sig;
    let func_name_fmt = func.sig.ident.to_string().to_kebab_case();
    is_known_keyword(&func_name_fmt)?;

    let mut content = String::new();
    write!(&mut content, "{}: function(", func_name_fmt)
        .context("cannot write function declaration in wit")?;
    let fn_args: Vec<String> = signature
        .inputs
        .iter()
        .map(|fn_arg| match fn_arg {
            syn::FnArg::Receiver(_) => bail!("does not support methods"),
            syn::FnArg::Typed(typed_pat) => {
                let pat = match &*typed_pat.pat {
                    syn::Pat::Ident(ident) => ident.ident.to_string().to_kebab_case(),
                    _ => bail!("can't handle this kind of fn argument"),
                };
                is_known_keyword(&pat)?;

                let ty = typed_pat.ty.to_wit()?;
                Ok(format!("{}: {}", pat, ty))
            }
        })
        .collect::<Result<Vec<String>>>()?;
    write!(&mut content, "{})", fn_args.join(", ")).context("cannot write end of func params")?;

    if let ReturnType::Type(_, return_ty) = &signature.output {
        if let Type::Tuple(tuple) = return_ty.as_ref() {
            let tuple_fields = tuple
                .elems
                .iter()
                .map(|f| f.to_wit())
                .collect::<Result<Vec<String>>>()?
                .join(", ");
            writeln!(&mut content, " -> tuple<{}>", tuple_fields).context("cannot write return type")?;
        } else {
            writeln!(&mut content, " -> {}", return_ty.to_wit()?)
                .context("cannot write return type")?;
        }
    }

    Ok(content)
}

/// Generate a wit type alias
/// ```rust
/// /// Document String
/// type foo = (String, option<bool>);
/// ```
/// becomes
/// ```ts
/// /// Document String
/// type foo = tuple<string, option<bool>>
/// ```
///
pub fn gen_wit_type_alias(type_alias: &ItemType) -> Result<String> {
    if !type_alias.generics.params.is_empty() {
        bail!("doesn't support generic parameters with witgen");
    }
    let ty = type_alias.ty.to_wit()?;
    let type_alias_ident = type_alias.ident.to_string().to_kebab_case();
    is_known_keyword(&type_alias_ident)?;

    Ok(format!("type {} = {}\n", type_alias_ident, ty))
}

pub(crate) fn get_doc_comment(attrs: &[Attribute]) -> Result<Option<String>> {
    let mut comment = String::new();
    for attr in attrs {
        match &attr.parse_meta()? {
            syn::Meta::NameValue(name_val) if name_val.path.is_ident("doc") => {
                if let Lit::Str(lit_str) = &name_val.lit {
                    writeln!(&mut comment, "/// {}", lit_str.value())?;
                }
            }
            _ => {}
        }
    }
    Ok((!comment.is_empty()).then(|| comment))
}
