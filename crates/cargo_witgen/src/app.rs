use anyhow::{bail, Context, Result};
use clap::Parser;
use std::{
    fs::{read, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use witgen_macro_helper::parse_crate_as_file;

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
        #[clap(long, short = 'i', default_value = "./src/lib.rs")]
        input: PathBuf,

        /// Specify output file to generate wit definitions
        #[clap(long, short = 'o', default_value = "index.wit")]
        output: PathBuf,

        /// Specify prefix file to copy into top of the generated wit file
        #[clap(long, short = 'p')]
        prefix_file: Vec<PathBuf>,

        /// Specify prefix string to copy into top of the generated wit file
        /// `--prefix-string 'use * from "string.wit"'`
        #[clap(long, short = 's')]
        prefix_string: Vec<String>,

        /// Print results to stdout instead of to file
        #[clap(long)]
        stdout: bool,
    },
}

impl App {
    pub fn run(&self) -> Result<()> {
        let Command::Generate {
            input,
            output,
            prefix_file,
            prefix_string,
            stdout,
        } = &self.command;
        if !input.exists() {
            bail!("input {:?} doesn't exist", input);
        }
        let wit = parse_crate_as_file(input)?;

        let mut wit_str = format!("// This is a generated file by witgen (https://github.com/bnjjj/witgen), please do not edit yourself, you can generate a new one thanks to cargo witgen generate command. (cargo-witgen v{}) \n\n", env!("CARGO_PKG_VERSION"));
        wit_str.push_str(&prefix_string.join("\n"));
        wit_str.push('\n');
        for path in prefix_file {
            let prefix_file = String::from_utf8(read(path)?)?;
            wit_str.push_str(&format!("{}\n\n", prefix_file));
        }
        wit_str.push_str(&wit.to_string());
        if *stdout {
            println!("{wit_str}");
        } else {
            write_file(output, &wit_str)?;
        }

        Ok(())
    }
}

fn write_file(path: &Path, contents: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .expect("cannot create file to generate wit");
    file.write_all(contents.as_bytes())
        .context("cannot write to file")?;
    file.flush().context("cannot flush file")?;
    Ok(())
}
