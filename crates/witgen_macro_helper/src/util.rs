use std::fmt::Display;

use anyhow::{bail, Result};
use heck::ToKebabCase;
use syn::{FnArg, ImplItem, ImplItemMethod, PatType, ReturnType, Signature, Type, Visibility};

use crate::wit::ToWitType;

pub enum FuncType {
    Instance(bool),
    Standalone,
}

pub fn non_receiver_args(fn_arg: &FnArg) -> Option<&PatType> {
    match fn_arg {
        FnArg::Typed(typed_pat) => Some(typed_pat),
        FnArg::Receiver(_) => None,
    }
}

/// Utilities for inspecting `Signature`s
pub trait SignatureUtils {
    fn fn_type(&self) -> FuncType;

    fn fn_args(&self) -> Result<Vec<String>>;

    fn ret_args(&self) -> Result<String>;
}

impl SignatureUtils for Signature {
    fn fn_type(&self) -> FuncType {
        for fn_arg in self.inputs.iter() {
            match fn_arg {
                FnArg::Receiver(r) => return FuncType::Instance(r.mutability.is_some()),
                FnArg::Typed(_) => continue,
            }
        }
        FuncType::Standalone
    }

    fn fn_args(&self) -> Result<Vec<String>> {
        self.inputs
            .iter()
            .filter_map(non_receiver_args)
            .map(|typed_pat| {
                let pat = match &*typed_pat.pat {
                    syn::Pat::Ident(ident) => wit_ident(&ident.ident)?,
                    _ => bail!("can't handle this kind of fn argument"),
                };
                let ty = typed_pat.ty.to_wit()?;
                Ok(format!("{}: {}", pat, ty))
            })
            .collect::<Result<Vec<String>>>()
    }

    fn ret_args(&self) -> Result<String> {
        let res = if let ReturnType::Type(_, return_ty) = &self.output {
            if let Type::Tuple(tuple) = return_ty.as_ref() {
                let tuple_fields = tuple
                    .elems
                    .iter()
                    .map(|f| f.to_wit())
                    .collect::<Result<Vec<String>>>()?
                    .join(", ");
                format!(" -> tuple<{}>", tuple_fields)
            } else {
                format!(" -> {}", return_ty.to_wit()?)
            }
        } else {
            "".to_string()
        };
        Ok(res)
    }
}

pub fn wit_ident<T: Display + ?Sized>(ident: &T) -> Result<String> {
    is_known_keyword(ident.to_string().to_kebab_case())
}

pub(crate) fn is_known_keyword(ident: String) -> Result<String> {
    if matches!(
        ident.as_str(),
        "use"
            | "type"
            | "resource"
            | "func"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "s8"
            | "s16"
            | "s32"
            | "s64"
            | "float32"
            | "float64"
            | "char"
            | "handle"
            | "record"
            | "enum"
            | "flags"
            | "variant"
            | "union"
            | "bool"
            | "string"
            | "option"
            | "list"
            | "expected"
            | "_"
            | "as"
            | "from"
            | "static"
            | "interface"
            | "tuple"
            | "async"
    ) {
        Err(anyhow::anyhow!(
            "'{}' is a known keyword you can't use the same identifier",
            ident
        ))
    } else {
        Ok(ident)
    }
}

pub fn pub_method(item: &ImplItem) -> Option<&ImplItemMethod> {
    match item {
        ImplItem::Method(
            method @ ImplItemMethod {
                vis: Visibility::Public(_),
                ..
            },
        ) => Some(method),
        _ => None,
    }
}
