use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author = "Benjamin Coenen <benjamin.coenen@hotmail.com>")]
pub struct Opt {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Generate wit files.
    /// 
    /// String options for identifiers and types:
    /// 
    /// "LOWER_CAMEL_CASE", "UPPER_CAMEL_CASE", "SNEK_CASE", "KEBAB_CASE" (default).
    #[clap(alias = "gen")]
    Generate {
        /// Specify output file to generate wit definitions
        #[clap(long, short = 'o')]
        output: Option<PathBuf>,

        /// Style of type names
        #[clap(long, short = 't')]
        types: Option<String>,

        /// Style of identifer names
        #[clap(long, short = 'i')]
        idents: Option<String>,

        /// Arguments to be passed to `cargo rustc ...`.
        #[clap(last = true)]
        args: Vec<String>,
    },
}
