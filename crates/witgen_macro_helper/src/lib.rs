#![deny(warnings)]
use std::path::Path;

use anyhow::{bail, Result};
use syn_file_expand::read_full_crate_source_code;

pub mod generator;
mod wit;
pub use wit::Wit;

/// Parse proc_macro2 tokens into Wit
pub fn parse_tokens(item: proc_macro2::TokenStream) -> Result<Wit> {
    item.try_into()
}

/// Read a crate starting from a single file then parse into Wit
pub fn parse_crate_as_file(path: &Path) -> Result<Wit> {
    if let Ok(ast) = read_full_crate_source_code(path, |_| Ok(false)) {
        Ok(ast.into())
    } else {
        bail!("Failed to parse crate source {:?}", path)
    }
}
