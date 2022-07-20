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
mod util;
pub mod visitor;

/// Convence function for
/// ```
/// let wit: Result<Wit> = tokens.try_into()
/// ```
pub fn parse_tokens(tokens: proc_macro2::TokenStream) -> Result<Wit> {
    tokens.try_into()
}

pub fn resolve_wit_file(root: &Path, name: &str) -> Result<(PathBuf, String)> {
    let wit = root.join(name).with_extension("wit");

    // Attempt to read a ".wit" file.
    match fs::read_to_string(&wit) {
        Ok(contents) => Ok((wit, contents)),

        // If no such file was found, attempt to read a ".wit.md" file.
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let wit_md = wit.with_extension("wit.md");
            match fs::read_to_string(&wit_md) {
                Ok(contents) => Ok((wit_md, contents)),
                Err(_err) => Err(err.into()),
            }
        }

        Err(err) => Err(err.into()),
    }
}

pub fn resolve_wit_files(root_paths: &[PathBuf], name: &str) -> Result<(PathBuf, String)> {
    for path in root_paths {
        if let Ok(res) = resolve_wit_file(path, name) {
            return Ok(res);
        }
    }
    bail!("Failed to resolve {name}")
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
        DefaultResolver {}.parse_wit_interface(name, wit_source)
    }
}

pub struct DefaultResolver;

impl Resolver for DefaultResolver {}
