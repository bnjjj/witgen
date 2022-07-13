//! This crate provides a way to parse a whole crate as a file and then parse this into a `Wit` type.
//! Currently this is a wrapper type for `syn` types.
//!
//!
//!
#![deny(warnings)]
use anyhow::{bail, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};
use syn::File;
pub use syn_file_expand::read_full_crate_source_code;
pub use wit_parser::Interface;

pub mod generator;
mod wit;
pub use wit::Wit;

/// Convence function for
/// ```
/// let wit: Result<Wit> = tokens.try_into()
/// ```
pub fn parse_tokens(tokens: proc_macro2::TokenStream) -> Result<Wit> {
    tokens.try_into()
}

/// Read a crate starting from a single file then parse into a file
pub fn parse_crate_as_file(path: &Path) -> Result<File> {
    if let Ok(file) = read_full_crate_source_code(path, |_| Ok(false)) {
        Ok(file)
    } else {
        bail!("Failed to parse crate source {:?}", path)
    }
}

/// Convence function for
/// ```
/// let wit: Wit = file.into();
/// ```
pub fn parse_file(file: File) -> Wit {
    file.into()
}

pub trait Resolver {
    fn resolve_name(&mut self, name: &str) -> Result<String> {
        bail!("Failed to resolev {name}")
    }

    fn parse_wit_interface(&mut self, name: &str, wit_source: &str) -> Result<Interface> {
        Interface::parse_with(name, wit_source, |name| {
            Ok((name.into(), self.resolve_name(name)?))
        })
    }

    fn parse_wit_interface_default(name: &str, wit_source: &str) -> Result<Interface> {
      Empty{}.parse_wit_interface(name, wit_source)
    }
}

pub fn parse_interface_from_wit<R: Resolver>(
    name: &str,
    wit_source: &str,
    resolver: &mut R,
) -> Result<Interface> {
    Interface::parse_with(name, wit_source, |name| {
        Ok((name.into(), resolver.resolve_name(name)?))
    })
}

struct Empty;

impl Resolver for Empty{}