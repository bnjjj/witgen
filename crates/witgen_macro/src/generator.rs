use std::{
    fmt::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use cargo_metadata::MetadataCommand;
use syn::{ItemEnum, ItemFn, ItemStruct, ItemType, ReturnType, Type};

use crate::{write_to_file, ToWitType};

pub(crate) fn get_target_dir() -> PathBuf {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("cannot fetch cargo metadata");

    metadata.target_directory.join("witgen").into()
}

pub(crate) fn gen_wit_struct(target_dir: &Path, strukt: &ItemStruct) -> Result<()> {
    if !strukt.generics.params.is_empty() {
        bail!("doesn't support generic parameters with witgen");
    }

    let struct_name = &strukt.ident;
    let mut is_tuple_struct = false;
    let attrs = strukt
        .fields
        .iter()
        .map(|field| {
            let field_name = match &field.ident {
                Some(ident) => ident.to_string() + ": ",
                None => {
                    is_tuple_struct = true;
                    String::new()
                }
            };

            Ok(format!("{}{}", field_name, field.ty.to_wit()?))
        })
        .collect::<Result<Vec<String>>>()?;
    let attrs = if is_tuple_struct {
        attrs.join(", ")
    } else {
        attrs.join(",\n\t")
    };

    let content = if is_tuple_struct {
        format!("type {} = tuple<{}>", struct_name, attrs)
    } else {
        format!(
            r#"record {} {{
    {}
}}"#,
            struct_name, attrs
        )
    };

    write_to_file(target_dir, content)?;

    Ok(())
}

pub(crate) fn gen_wit_enum(target_dir: &Path, enm: &ItemEnum) -> Result<()> {
    if !enm.generics.params.is_empty() {
        bail!("doesn't support generic parameters with witgen");
    }

    let enm_name = &enm.ident;
    let variants = enm
        .variants
        .iter()
        .map(|variant| match &variant.fields {
            syn::Fields::Named(_named) => Err(anyhow::anyhow!(
                "named variant fields are not already supported"
            )),
            syn::Fields::Unnamed(unamed) => {
                let fields = unamed
                    .unnamed
                    .iter()
                    .map(|field| field.ty.to_wit())
                    .collect::<Result<Vec<String>>>()?
                    .join(", ");
                Ok(format!("{}({}),", variant.ident.to_string(), fields))
            }
            syn::Fields::Unit => Ok(variant.ident.to_string() + ","),
        })
        .collect::<Result<Vec<String>>>()?
        .join("\n\t");
    let content = format!(
        r#"variant {} {{
    {}
}}"#,
        enm_name, variants
    );

    write_to_file(target_dir, content)?;

    Ok(())
}

pub(crate) fn gen_wit_function(target_dir: &Path, func: &ItemFn) -> Result<()> {
    let signature = &func.sig;
    let mut content = String::new();
    write!(&mut content, "{} : function(", func.sig.ident.to_string())
        .context("cannot write function declaration in wit")?;
    let fn_args: Vec<String> = signature
        .inputs
        .iter()
        .map(|fn_arg| match fn_arg {
            syn::FnArg::Receiver(_) => bail!("does not support methods"),
            syn::FnArg::Typed(typed_pat) => {
                let pat = match &*typed_pat.pat {
                    syn::Pat::Ident(ident) => ident.ident.to_string(),
                    _ => bail!("can't handle this kind of fn argument"),
                };
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

    write_to_file(target_dir, content)?;

    Ok(())
}

pub(crate) fn gen_wit_type_alias(target_dir: &Path, type_alias: &ItemType) {
    todo!()
}
