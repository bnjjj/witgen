use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use clap_cargo_extra::ClapCargo;
use heck::ToKebabCase;
use regex::Regex;
use std::{
    collections::HashMap,
    // fmt::Write,
    fs::{read, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use syn::File;
use witgen_macro_helper::{parse_crate_as_file, Resolver, Wit};

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
    /// Specify input file to generate wit definitions from
    #[clap(long, short = 'i')]
    pub input: Option<PathBuf>,

    /// Specify input directory to generate wit definitions from
    ///
    ///
    /// Will expect library: `<input-dir>/src/lib.rs`
    #[clap(long, short = 'd', default_value = ".")]
    pub input_dir: PathBuf,

    /// Specify output file to generate wit definitions
    #[clap(long, short = 'o', default_value = "index.wit")]
    pub output: PathBuf,

    /// Specify prefix file to copy into top of the generated wit file
    #[clap(long, short = 'b')]
    pub prefix_file: Vec<PathBuf>,

    /// Specify prefix string to copy into top of the generated wit file
    /// `--prefix-string 'use * from "string.wit"'`
    #[clap(long, short = 'a')]
    pub prefix_string: Vec<String>,

    /// Print results to stdout instead file
    #[clap(long)]
    pub stdout: bool,

    /// Do not resolve the `use` references in generated wit file to combine into one
    #[clap(long)]
    pub skip_resolve: bool,

    /// Skip adding prologue to file
    #[clap(long)]
    pub skip_prologue: bool,

    #[clap(flatten)]
    pub cargo: ClapCargo,
}

impl Witgen {
    pub fn from_path(path: &Path) -> Self {
        Self {
            input: None,
            input_dir: path.to_path_buf(),
            output: PathBuf::from("index.wit"),
            prefix_file: vec![],
            prefix_string: vec![],
            stdout: false,
            cargo: ClapCargo::default(),
            skip_resolve: false,
            skip_prologue: true,
        }
    }

    pub fn gen_from_path(path: &Path) -> Result<String> {
        let witgen = Witgen::from_path(path);
        witgen.generate_str(witgen.read_input()?)
    }

    // Part of extra API but current results in unused warning
    #[allow(dead_code)]
    pub fn gen_static_from_path(path: &Path) -> Result<String> {
        let witgen = Witgen::from_path(path);
        witgen.resolve(&witgen.generate_str(witgen.read_input()?)?)
    }

    pub fn read_input(&self) -> Result<File> {
        // TODO: figure out how to avoid the clone()
        let input = self
            .input
            .as_ref()
            .map_or_else(|| self.input_dir.join("src/lib.rs"), |i| i.clone());

        if !input.exists() {
            bail!("input {:?} doesn't exist", input);
        }
        parse_crate_as_file(&input)
    }

    pub fn generate_str(&self, file: File) -> Result<String> {
        let wit: Wit = file.into();
        let mut wit_str = if self.skip_prologue {
            String::new()
        } else {
            format!("// auto-generated file by witgen (https://github.com/bnjjj/witgen), please do not edit yourself, you can generate a new one thanks to cargo witgen generate command. (cargo-witgen v{}) \n\n", env!("CARGO_PKG_VERSION"))
        };
        if !self.prefix_string.is_empty() {
            wit_str.push_str(&self.prefix_string.join("\n"));
            wit_str.push('\n');
        }
        for path in &self.prefix_file {
            let prefix_file = String::from_utf8(read(path)?)?;
            wit_str.push_str(&prefix_file);
            wit_str.push('\n');
        }
        wit_str.push_str(&wit.to_string());
        Ok(wit_str)
    }

    pub fn write_output(&self, wit_str: &str) -> Result<()> {
        if self.stdout {
            println!("{wit_str}");
        } else {
            write_file(&self.output, wit_str)?;
        }
        Ok(())
    }

    pub fn resolve_wit(&self, wit_str: &str) -> Result<HashMap<String, String>> {
        let mut resolver = WitResolver::new(&self.cargo);
        let _ = resolver.parse_wit_interface(
            self.output.to_str().expect("failed to decode output"),
            wit_str,
        )?;
        Ok(resolver.wit_generated)
    }

    pub fn run(&self) -> Result<()> {
        let input = self.read_input()?;
        let mut wit_str = self.generate_str(input)?;
        if !self.skip_resolve {
            wit_str = self.resolve(&wit_str)?;
        }
        self.write_output(&wit_str)
    }

    pub fn resolve(&self, wit_str: &str) -> Result<String> {
        let dep_wit = self
            .resolve_wit(wit_str)?
            .into_values()
            .collect::<Vec<String>>()
            .join("\n");

        // remove `use` from file since combining
        let re = Regex::new(r"^use .+\n").unwrap();
        let mut res = re.replace_all(wit_str, "").to_string();
        res.push_str(&dep_wit);
        Ok(res)
    }
}

struct WitResolver<'a> {
    cargo: &'a ClapCargo,
    wit_generated: HashMap<String, String>,
}

impl<'a> WitResolver<'a> {
    fn new(cargo: &'a ClapCargo) -> Self {
        Self {
            cargo,
            wit_generated: Default::default(),
        }
    }
}

impl Resolver for WitResolver<'_> {
    fn resolve_name(&mut self, name: &str) -> Result<String> {
        // TODO: Handle package names that have hyphen. e.g. `near_sdk` --> `near-sdk`
        // let mut package = ;
        let package = self
            .cargo
            .find_package(name)?
            .or_else(|| self.cargo.find_package(&name.to_kebab_case()).unwrap_or(None))
            .map_or_else(|| bail!("Failed to find {name}"), Ok)?;

        let manifest_dir = package.manifest_path.as_std_path().parent().map_or_else(
            || bail!("failed to find parent of {}", package.manifest_path),
            Ok,
        )?;

        let res = Witgen::gen_from_path(manifest_dir)?;
        self.wit_generated.insert(name.to_string(), res.clone());
        Ok(res)
    }
}

impl Command {
    pub fn run(&self) -> Result<()> {
        match self {
            Command::Generate(witgen) => witgen.run()?,
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
