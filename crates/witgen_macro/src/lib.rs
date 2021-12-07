use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use generator::{gen_wit_enum, gen_wit_function, gen_wit_struct, get_target_dir};
use once_cell::sync::OnceCell;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use syn::{parse, ItemEnum, ItemFn, ItemStruct, Type};

mod generator;

static TARGET_PATH: OnceCell<PathBuf> = OnceCell::new();

macro_rules! handle_error {
    ($op: expr) => {
        if let Err(err) = $op {
            return syn::Error::new(Span::call_site(), format!("witgen error: {}", err))
                .to_compile_error()
                .into();
        };
    };
}

#[proc_macro_attribute]
pub fn witgen(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let target_dir = TARGET_PATH.get_or_init(get_target_dir);
    if !target_dir.exists() {
        fs::create_dir_all(target_dir).expect("cannot create target dir");
    }

    let strukt = parse::<ItemStruct>(item.clone());
    if let Ok(strukt) = &strukt {
        handle_error!(gen_wit_struct(target_dir, strukt));
        return item;
    }

    let func = parse::<ItemFn>(item.clone());
    if let Ok(func) = &func {
        handle_error!(gen_wit_function(target_dir, func));
        return item;
    }

    let enm = parse::<ItemEnum>(item.clone());
    if let Ok(enm) = &enm {
        handle_error!(gen_wit_enum(target_dir, enm));
        return item;
    }

    // TODO add enum

    syn::Error::new_spanned(
        proc_macro2::TokenStream::from(item),
        "Cannot put wit_generator proc macro on this kind of element",
    )
    .to_compile_error()
    .into()
}

trait ToWitType {
    fn to_wit(&self) -> Result<String>;
}

impl ToWitType for Type {
    fn to_wit(&self) -> Result<String> {
        let res = match self {
            Type::Array(array) => {
                format!("list <{}>", array.elem.to_wit()?)
            }
            Type::Slice(array) => {
                format!("list <{}>", array.elem.to_wit()?)
            }
            Type::Path(path) => {
                let last_path_seg = path.path.segments.last().ok_or_else(|| {
                    anyhow::anyhow!(
                        "cannot get type path segment for type '{}'",
                        self.to_token_stream()
                    )
                })?;
                let global_ty = last_path_seg.ident.to_string();
                match global_ty.as_str() {
                    // Add Box/ARC/RC ?
                    wrapper_ty @ ("Vec" | "Option") => match &last_path_seg.arguments {
                        syn::PathArguments::AngleBracketed(generic_args) => {
                            if generic_args.args.len() > 1 {
                                bail!("generic args of {} should not be more than 1", wrapper_ty);
                            }
                            match generic_args.args.first().unwrap() {
                                syn::GenericArgument::Type(ty) => {
                                    let new_ty_name = match wrapper_ty {
                                        "Vec" => "list",
                                        "Option" => "option",
                                        _ => unreachable!(),
                                    };
                                    format!("{} <{}>", new_ty_name, ty.to_wit()?)
                                }
                                other => {
                                    bail!("generic args type {:?} is not implemented", other)
                                }
                            }
                        }
                        syn::PathArguments::Parenthesized(_) | syn::PathArguments::None => {
                            bail!("parenthized path argument is not implemented")
                        }
                    },
                    "String" => "string".to_string(),
                    _ => {
                        let ident = path.path.get_ident().ok_or_else(|| {
                            anyhow::anyhow!("cannot get identifier for a type '{}', type who takes generics are not currently supported", self.to_token_stream())
                        })?;
                        match ident.to_string().as_str() {
                            ident @ ("i8" | "i16" | "i32" | "i64") => {
                                format!("s{}", ident.trim_start_matches('i'))
                            }
                            "usize" => String::from("u64"),
                            "isize" => String::from("i64"),
                            ident => ident.to_string(),
                        }
                    }
                }
            }
            Type::Tuple(tuple) => {
                format!(
                    "tuple<{}>",
                    tuple
                        .elems
                        .iter()
                        .map(|ty| ty.to_wit())
                        .collect::<Result<Vec<String>>>()?
                        .join(", ")
                )
            }

            _ => bail!(
                "cannot serialize this type '{}' to wit",
                self.to_token_stream()
            ),
        };

        Ok(res)
    }
}

pub(crate) fn hash_string(query: &str) -> String {
    use sha2::{Digest, Sha256};

    hex::encode(Sha256::digest(query.as_bytes()))
}

pub(crate) fn write_to_file(target_dir: &Path, content: String) -> Result<()> {
    if std::env::var("WITGEN_ENABLED").map(|v| v.to_lowercase()) != Ok("true".to_string()) {
        return Ok(());
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(target_dir.join(hash_string(&content)).with_extension("wit"))
        .context("cannot create file to generate wit")?;

    file.write_all(content.as_bytes())
        .context("cannot write to wit file")?;
    file.flush().context("cannot flush wit file")?;

    Ok(())
}
