use std::{path::PathBuf, env};
use anyhow::Result;
use clap::Parser;
use syn_file_expand::read_full_crate_source_code;
use witgen_macro_helper::Wit;

#[derive(Parser, Debug)]
#[clap(author = "Benjamin Coenen <benjamin.coenen@hotmail.com>")]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Generate wit files
    #[clap(alias = "gen")]
    Generate {
        /// Specify output file to generate wit definitions
        #[clap(long, short = 'o')]
        output: Option<PathBuf>,

        /// Arguments to be passed to `cargo rustc ...`.
        #[clap(last = true)]
        args: Vec<String>,

        /// Specify prefix file to copy into top of the generated wit file
        #[clap(long, short = 'p')]
        prefix_file: Option<PathBuf>,

        /// Specify prefix string to copy into top of the generated wit file
        /// `--prefix-string 'use * from "string.wit"'`
        #[clap(long, short = 's')]
        prefix_string: Option<String>,
    },
}


impl App {
  pub fn run(&self) -> Result<()> {
    
    let ast = read_full_crate_source_code(env::current_dir()?.join("./src/lib.rs"), |_|Ok(false)).unwrap();
    println!("{}", Wit::from_file(ast));

    Ok(())
  }
}