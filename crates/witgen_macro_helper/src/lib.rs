#![deny(warnings)]

use anyhow::{Result};


mod generator;

mod wit;
pub use wit::Wit;


pub fn parse_str(s: &str) -> Result<String> {
    parse_tokens(syn::parse_str::<proc_macro2::TokenStream>(s)?)
}

pub fn parse_tokens(item: proc_macro2::TokenStream) -> Result<String> {
    Ok(format!("{}", Wit::try_parse(item)?))
}
