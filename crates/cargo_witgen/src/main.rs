#![deny(warnings)]
#![doc = include_str!("../README.md")]

use anyhow::{Context, Result};
use clap::{crate_version, FromArgMatches, IntoApp};

use std::env;

mod app;
use crate::app::App;

fn main() -> Result<()> {
    let args = env::args_os().skip(2);

    let matches = App::command()
        .version(crate_version!())
        .bin_name("cargo witgen")
        .no_binary_name(true)
        .get_matches_from(args);

    App::from_arg_matches(&matches)
        .context("Command not found")?
        .run()
}
