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
    /// Generate wit files
    #[clap(alias = "gen")]
    Generate {
        /// Specify output file to generate wit definitions
        #[clap(long, short = 'o')]
        output: Option<PathBuf>,

        /// Arguments to be passed to `cargo rustc ...`.
        #[clap(last = true)]
        args: Vec<String>,
    },
}
