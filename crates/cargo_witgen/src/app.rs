use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use std::{
    fs::{read, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use witgen_macro_helper::parse_crate_as_file;

#[derive(Parser, Debug)]
#[clap(
    author = "Benjamin Coenen <benjamin.coenen@hotmail.com>, Willem Wyndham <willem@ahalabs.dev>"
)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate wit files
    #[clap(alias = "gen")]
    Generate(Witgen),
}

#[derive(Debug, Args)]
pub struct Witgen {
    /// Specify output file to generate wit definitions
    #[clap(long, short = 'i', default_value = "./src/lib.rs")]
    pub input: PathBuf,

    /// Specify output file to generate wit definitions
    #[clap(long, short = 'o', default_value = "index.wit")]
    pub output: PathBuf,

    /// Specify prefix file to copy into top of the generated wit file
    #[clap(long, short = 'p')]
    pub prefix_file: Vec<PathBuf>,

    /// Specify prefix string to copy into top of the generated wit file
    /// `--prefix-string 'use * from "string.wit"'`
    #[clap(long, short = 's')]
    pub prefix_string: Vec<String>,

    /// Print results to stdout instead of to file
    #[clap(long)]
    pub stdout: bool,
}

impl Witgen {
    pub fn generate_str(&self) -> Result<String> {
        let Witgen {
            input,
            prefix_file,
            prefix_string,
            ..
        } = self;
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

        Ok(wit_str)
    }

    pub fn output_wit_str(&self, wit_str: &str) -> Result<()> {
        if self.stdout {
            println!("{wit_str}");
        } else {
            write_file(&self.output, wit_str)?;
        }
        Ok(())
    }

    pub fn generate_and_output(&self) -> Result<()> {
        self.output_wit_str(&self.generate_str()?)
    }
}

impl Command {
    pub fn run(&self) -> Result<()> {
        match self {
            Command::Generate(witgen) => witgen.generate_and_output()?,
        };
        Ok(())
    }
}

impl App {
    #[allow(dead_code)]
    pub fn run(&self) -> Result<()> {
        self.command.run()
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
