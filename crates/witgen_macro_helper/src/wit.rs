use std::fmt::Display;
use std::str::FromStr;

use crate::generator::{
    gen_wit_enum, gen_wit_function, gen_wit_struct, gen_wit_type_alias, get_doc_comment,
};
use anyhow::{bail, Result};
use heck::ToKebabCase;
use quote::ToTokens;
use syn::{
    parse2 as parse, Attribute, File, Item, ItemEnum, ItemFn, ItemMod, ItemStruct, ItemType,
    Type as SynType, TypeReference,
};

/// Wit type that correspond to Rust Types using `syn`'s representation
pub enum Wit {
    Mod(Vec<Wit>, Vec<Attribute>),
    Record(ItemStruct),
    Function(ItemFn),
    Variant(ItemEnum),
    Type(ItemType),
}

impl Wit {
    fn from_items(items: Vec<Item>) -> Vec<Self> {
        items
            .into_iter()
            .filter_map(|item| item.try_into().ok())
            .collect()
    }

    pub fn attrs(&self) -> Option<&[Attribute]> {
        match self {
            Wit::Record(item) => Some(&item.attrs),
            Wit::Function(item) => Some(&item.attrs),
            Wit::Variant(item) => Some(&item.attrs),
            Wit::Type(item) => Some(&item.attrs),
            Wit::Mod(_, attrs) => Some(attrs),
        }
    }

    pub fn get_doc(&self) -> Result<Option<String>> {
        get_doc_comment(self.attrs().unwrap_or_default())
    }

    pub fn validate(self) -> Result<Self> {
        use Wit::*;
        match self {
            Mod(_, _) => Ok(self),
            other if has_witgen_macro(&self.attrs()) => Ok(other),
            _ => bail!("Has no witgen macro"),
        }
    }
}

fn has_witgen_macro(attrs: &Option<&[Attribute]>) -> bool {
    attrs.map_or(false, |attrs| {
        for attr in attrs.iter() {
            if is_witgen_macro(attr) {
                return true;
            }
        }
        false
    })
}

fn is_witgen_macro(attr: &Attribute) -> bool {
    // TODO: make this not use string comparison.
    format!("{:#?}", attr.path).contains("witgen")
}

impl From<File> for Wit {
    fn from(file: File) -> Self {
        Wit::Mod(Wit::from_items(file.items), vec![])
    }
}

impl TryFrom<Item> for Wit {
    type Error = anyhow::Error;

    fn try_from(item: Item) -> Result<Self, Self::Error> {
        match item {
            Item::Enum(item) => Wit::Variant(item),
            Item::Fn(item) => Wit::Function(item),
            Item::Struct(item) => Wit::Record(item),
            Item::Type(item) => Wit::Type(item),
            Item::Mod(ItemMod {
                content: Some((_, items)),
                attrs,
                ..
            }) => Wit::Mod(Wit::from_items(items), attrs),
            _ => bail!("cannot prase item"),
        }
        .validate()
    }
}

impl TryFrom<proc_macro2::TokenStream> for Wit {
    type Error = anyhow::Error;

    fn try_from(item: proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        if let Ok(file) = parse::<File>(item.clone()) {
            Ok(file.into())
        } else if let Ok(item) = parse::<Item>(item.clone()) {
            Wit::try_from(item)
        } else {
            bail!(
                "Cannot put witgen proc macro on this kind of item: {}",
                item
            )
        }?
        .validate()
    }
}

impl FromStr for Wit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl TryFrom<&str> for Wit {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        syn::parse_str::<proc_macro2::TokenStream>(s)?.try_into()
    }
}

impl Display for Wit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let doc = self.get_doc().unwrap_or(None).unwrap_or_default();
        let wit_str = match self {
            Wit::Mod(wit, _) => Ok(wit
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join("\n")),
            Wit::Record(item) => gen_wit_struct(item),
            Wit::Function(item) => gen_wit_function(item),
            Wit::Variant(item) => gen_wit_enum(item),
            Wit::Type(item) => gen_wit_type_alias(item),
        }
        .unwrap_or_default();
        write!(f, "{doc}{wit_str}")
    }
}

pub(crate) trait ToWitType {
    fn to_wit(&self) -> Result<String>;
}

impl ToWitType for SynType {
    fn to_wit(&self) -> Result<String> {
        let res = match self {
            SynType::Array(array) => {
                format!("list<{}>", array.elem.to_wit()?)
            }
            SynType::Slice(array) => {
                format!("list<{}>", array.elem.to_wit()?)
            }
            SynType::Path(path) => {
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
                                    format!("{}<{}>", new_ty_name, ty.to_wit()?)
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
                    wrapper_ty @ "HashMap" => match &last_path_seg.arguments {
                        syn::PathArguments::AngleBracketed(generic_args) => {
                            if generic_args.args.len() != 2 {
                                bail!("generic args of {} should be 2", wrapper_ty);
                            }

                            let args = generic_args
                                .args
                                .iter()
                                .map(|arg| match arg {
                                    syn::GenericArgument::Type(ty) => ty.to_wit(),
                                    other => {
                                        bail!("generic args type {:?} is not implemented", other)
                                    }
                                })
                                .collect::<Result<Vec<String>>>()?;
                            format!("list<tuple<{}>>", args.join(","))
                        }
                        syn::PathArguments::Parenthesized(_) | syn::PathArguments::None => {
                            bail!("parenthized path argument is not implemented")
                        }
                    },
                    wrapper_ty @ "Result" => match &last_path_seg.arguments {
                        syn::PathArguments::AngleBracketed(generic_args) => {
                            if generic_args.args.len() > 2 {
                                bail!("generic args of {} should not be more than 2", wrapper_ty);
                            }
                            let generic_args = generic_args
                                .args
                                .iter()
                                .map(|t| match t {
                                    syn::GenericArgument::Type(ty) => ty.to_wit(),
                                    other => Err(anyhow::anyhow!(
                                        "generic args type {:?} is not implemented",
                                        other
                                    )),
                                })
                                .collect::<Result<Vec<String>>>()?;

                            format!("expected<{}>", generic_args.join(", "))
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
                            ident @ ("f32" | "f64" | "u8" | "u16" | "u32" | "u64" | "char"
                            | "bool") => ident.to_string(),
                            ident => {
                                let ident_formatted = ident.to_string().to_kebab_case();
                                is_known_keyword(&ident_formatted)?;

                                ident_formatted
                            }
                        }
                    }
                }
            }
            SynType::Tuple(tuple) => {
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
            SynType::Reference(r) => {
                let TypeReference { elem, .. } = r;
                return elem.to_wit();
            }
            _ => bail!(
                "cannot serialize this type '{}' to wit",
                self.to_token_stream()
            ),
        };

        Ok(res)
    }
}

pub(crate) fn is_known_keyword(ident: &str) -> Result<()> {
    if matches!(
        ident,
        "use"
            | "type"
            | "resource"
            | "function"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "s8"
            | "s16"
            | "s32"
            | "s64"
            | "f32"
            | "f64"
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
        Ok(())
    }
}
