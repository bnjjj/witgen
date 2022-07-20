use std::fmt::Write;

use anyhow::{bail, Context, Result};
use heck::ToKebabCase;
use syn::{
    Attribute, Field, Fields, Ident, ItemEnum, ItemFn, ItemStruct, ItemTrait, ItemType, ItemUse,
    Lit, ReturnType, Signature, TraitItem, Type, UsePath, UseTree,
};

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

    let struct_name = gen_wit_ident(&strukt.ident)?;
    let is_tuple_struct = strukt.fields.iter().any(|f| f.ident.is_none());
    let fields = gen_fields(strukt.fields.iter().collect::<Vec<&Field>>())?;
    let fields = if is_tuple_struct {
        fields.join(", ")
    } else {
        fields.join(",\n")
    };

    let content = if is_tuple_struct {
        format!("type {} = tuple<{}>\n", struct_name, fields)
    } else {
        format!(
            r#"record {} {{
{}
}}
"#,
            struct_name, fields
        )
    };
    Ok(content)
}

fn gen_fields(iter: Vec<&Field>) -> Result<Vec<String>> {
    iter.into_iter()
        .map(|field| {
            let field_name = if let Some(ident) = &field.ident {
                format!("  {}: ", gen_wit_ident(ident)?)
            } else {
                Default::default()
            };
            let comment = get_doc_comment(&field.attrs, 1, false)?;
            Ok(format!("{comment}{}{}", field_name, field.ty.to_wit()?))
        })
        .collect()
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

    let enm_name = gen_wit_ident(&enm.ident)?;
    let is_wit_enum = enm
        .variants
        .iter()
        .all(|v| matches!(v.fields, Fields::Unit));
    let mut named_types = String::new();
    let variants = enm
        .variants
        .iter()
        .map(|variant| {
            let ident = gen_wit_ident(&variant.ident)?;
            let comment = get_doc_comment(&variant.attrs, 1, false)?;
            let variant_string = match &variant.fields {
                syn::Fields::Named(_named) => {
                    let fields = gen_fields(_named.named.iter().collect())?.join(",\n");
                    let inner_type_name = &format!("{}-{}", enm_name, ident);
                    let comment = get_doc_comment(&variant.attrs, 0, false)?;
                    write!(
                        &mut named_types,
                        "{}record {} {{\n{}\n}}\n",
                        comment, inner_type_name, fields
                    )?;
                    Ok(format!("{}({})", ident, inner_type_name))
                }
                syn::Fields::Unnamed(unamed) => {
                    let fields = unamed
                        .unnamed
                        .iter()
                        .map(|field| field.ty.to_wit())
                        .collect::<Result<Vec<String>>>()?
                        .join(", ");

                    let formatted_field = if unamed.unnamed.len() > 1 {
                        format!("tuple<{}>", fields)
                    } else {
                        fields
                    };

                    Ok(format!("{}({})", ident, formatted_field))
                }
                syn::Fields::Unit => Ok(ident),
            };
            variant_string.map(|v| format!("{}  {},", comment, v))
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

    Ok(format!("{}{}", content, named_types))
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
    gen_wit_function_from_signature(signature)
}

fn gen_wit_function_from_signature(signature: &Signature) -> Result<String> {
    let func_name_fmt = gen_wit_ident(&signature.ident);
    is_known_keyword(&func_name_fmt)?;

    let mut content = String::new();
    write!(&mut content, "{}: func(", func_name_fmt)
        .context("cannot write function declaration in wit")?;
    let fn_args: Vec<String> = signature
        .inputs
        .iter()
        .map(|fn_arg| match fn_arg {
            syn::FnArg::Receiver(_) => bail!("does not support methods"),
            syn::FnArg::Typed(typed_pat) => {
                let pat = match &*typed_pat.pat {
                    syn::Pat::Ident(ident) => gen_wit_ident(&ident.ident),
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
            writeln!(&mut content, " -> tuple<{}>", tuple_fields)
                .context("cannot write return type")?;
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
    let type_alias_ident = gen_wit_ident(&type_alias.ident)?;
    Ok(format!("type {} = {}\n", type_alias_ident, ty))
}

pub(crate) fn get_doc_comment(attrs: &[Attribute], depth: usize) -> Result<Option<String>> {
    let mut comment = String::new();
    for attr in attrs {
        match &attr.parse_meta()? {
            syn::Meta::NameValue(name_val) if name_val.path.is_ident("doc") => {
                if let Lit::Str(lit_str) = &name_val.lit {
                    writeln!(
                        &mut comment,
                        "{}///{}",
                        " ".repeat(depth * 2),
                        lit_str.value()
                    )?;
                }
            }
            _ => {}
        }
    }
    Ok((!comment.is_empty()).then(|| comment))
}

pub fn gen_wit_import(import: &ItemUse) -> Result<String> {
    let import_name = match &import.tree {
        UseTree::Path(UsePath { ident, .. }) => gen_wit_ident(ident)?,
        UseTree::Name(_) => todo!(),
        UseTree::Rename(_) => todo!(),
        UseTree::Glob(_) => todo!(),
        UseTree::Group(_) => todo!(),
    };
    // Todo allow referencing specific items
    Ok(format!("use * from {import_name}"))
}

pub fn gen_wit_trait(trait_: &ItemTrait) -> Result<String> {
    let name = gen_wit_ident(&trait_.ident)?;
    let mut res = format!("interface {name} {{\n");

    for item in trait_.items.iter() {
        match item {
            TraitItem::Const(_) => todo!("Const in Trait isn't implemented yet"),
            TraitItem::Method(method) => {
                let comment = get_doc_comment(&method.attrs, 1)?;
                write!(
                    &mut res,
                    "{comment}{}",
                    gen_wit_function_from_signature(&method.sig)?
                )?
            }
            TraitItem::Type(_) => todo!("Type in Trait isn't implemented yet"),
            TraitItem::Macro(_) => todo!("Macro in Trait isn't implemented yet"),
            TraitItem::Verbatim(_) => todo!("Verbatim in Trait isn't implemented yet"),
            _ => todo!("extra case in Trait isn't implemented yet"),
        }
    }
    res.push_str("}\n");
    Ok(res)
}

pub fn gen_wit_ident<T: Display + ?Sized>(ident: &T) -> Result<String> {
    is_known_keyword(ident.to_string().to_kebab_case())
}

pub fn gen_wit_impl(impl_: &ItemImpl) -> Result<String> {
    let name = gen_wit_ident(&impl_.self_ty.to_wit()?)?;
    let comment = get_doc_comment(&impl_.attrs, 0)?;
    let mut res = format!("{comment}resource {name} {{\n");

    for item in impl_.items.iter() {
        match item {
            ImplItem::Const(_) => todo!("Const in Impl isn't implemented yet"),
            ImplItem::Method(method) => {
                let comment = get_doc_comment(&method.attrs, 1)?;
                write!(
                    &mut res,
                    "{comment}{}",
                    gen_wit_function_from_signature(&method.sig)?
                )?
            }
            ImplItem::Type(_) => todo!("Type in Impl isn't implemented yet"),
            ImplItem::Macro(_) => todo!("Macro in Impl isn't implemented yet"),
            ImplItem::Verbatim(_) => todo!("Verbatim in Impl isn't implemented yet"),
            _ => todo!("extra case in Impl isn't implemented yet"),
        }
    }
    res.push_str("}\n");
    Ok(res)
}

