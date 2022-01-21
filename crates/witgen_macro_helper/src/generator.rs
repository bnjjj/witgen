use std::{fmt::Write, path::PathBuf};

use anyhow::{bail, Context, Result};
use cargo_metadata::MetadataCommand;
use heck::ToKebabCase;
use syn::{Attribute, ItemEnum, ItemFn, ItemStruct, ItemType, Lit, ReturnType, Type};

use crate::{is_known_keyword, ToWitType};

pub fn get_target_dir() -> PathBuf {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("cannot fetch cargo metadata");

    metadata.target_directory.join("witgen").into()
}

pub fn gen_wit_struct(strukt: &ItemStruct) -> Result<String> {
    if !strukt.generics.params.is_empty() {
        bail!("doesn't support generic parameters with witgen");
    }

    let struct_name = strukt.ident.to_string().to_kebab_case();
    is_known_keyword(&struct_name)?;

    let mut is_tuple_struct = false;
    let comment = get_doc_comment(&strukt.attrs)?;
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
                Some(comment) => Ok(format!("{}\t{}", comment, field_wit)),
                None => Ok(field_wit),
            }
        })
        .collect::<Result<Vec<String>>>()?;
    let attrs = if is_tuple_struct {
        attrs.join(", ")
    } else {
        attrs.join(",\n\t")
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

    match comment {
        Some(comment) => Ok(format!("{}{}", comment, content)),
        None => Ok(content),
    }
}

pub fn gen_wit_enum(enm: &ItemEnum) -> Result<String> {
    if !enm.generics.params.is_empty() {
        bail!("doesn't support generic parameters with witgen");
    }

    let enm_name = enm.ident.to_string().to_kebab_case();
    is_known_keyword(&enm_name)?;

    let comment = get_doc_comment(&enm.attrs)?;
    let variants = enm
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            syn::Fields::Named(_named) => Err(anyhow::anyhow!(
                "named variant fields are not already supported"
            )),
            syn::Fields::Unnamed(unamed) => {
                let comment = get_doc_comment(&variant.attrs)?;
                let fields = unamed
                    .unnamed
                    .iter()
                    .map(|field| field.ty.to_wit())
                    .collect::<Result<Vec<String>>>()?
                    .join(", ");
                let variant_ident = variant.ident.to_string().to_kebab_case();
                is_known_keyword(&variant_ident)?;

                let variant_wit = if unamed.unnamed.len() > 1 {
                    format!("{}(tuple<{}>),", variant_ident, fields)
                } else {
                    format!("{}({}),", variant_ident, fields)
                };

                match comment {
                    Some(comment) => Ok(format!("{}\t{}", comment, variant_wit)),
                    None => Ok(variant_wit),
                }
            }
            syn::Fields::Unit => {
                let comment = get_doc_comment(&variant.attrs)?;
                let variant_wit = variant.ident.to_string().to_kebab_case() + ",";

                match comment {
                    Some(comment) => Ok(format!("{}\t{}", comment, variant_wit)),
                    None => Ok(variant_wit),
                }
            }
        })
        .collect::<Result<Vec<String>>>()?
        .join("\n\t");
    let content = format!(
        r#"variant {} {{
    {}
}}
"#,
        enm_name, variants
    );

    match comment {
        Some(comment) => Ok(format!("{}{}", comment, content)),
        None => Ok(content),
    }
}

pub fn gen_wit_function(func: &ItemFn) -> Result<String> {
    let signature = &func.sig;
    let comment = get_doc_comment(&func.attrs)?;
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
            writeln!(&mut content, " -> ({})", tuple_fields).context("cannot write return type")?;
        } else {
            writeln!(&mut content, " -> {}", return_ty.to_wit()?)
                .context("cannot write return type")?;
        }
    }

    match comment {
        Some(comment) => Ok(format!("{}{}", comment, content)),
        None => Ok(content),
    }
}

pub fn gen_wit_type_alias(type_alias: &ItemType) -> Result<String> {
    if !type_alias.generics.params.is_empty() {
        bail!("doesn't support generic parameters with witgen");
    }
    let comment = get_doc_comment(&type_alias.attrs)?;
    let ty = type_alias.ty.to_wit()?;
    let type_alias_ident = type_alias.ident.to_string().to_kebab_case();
    is_known_keyword(&type_alias_ident)?;

    let content = format!("type {} = {}\n", type_alias_ident, ty);

    match comment {
        Some(comment) => Ok(format!("{}{}", comment, content)),
        None => Ok(content),
    }
}

fn get_doc_comment(attrs: &[Attribute]) -> Result<Option<String>> {
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
