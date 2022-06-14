//! This crate provides a way to parse a whole crate as a file and then parse this into a `Wit` type.
//! Currently this is a wrapper type for `syn` types.
//!
//!
//!
#![deny(warnings)]
use std::path::Path;

use anyhow::{bail, Result};
use syn::File;
pub use syn_file_expand::read_full_crate_source_code;
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
