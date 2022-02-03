#![deny(warnings)]
#![doc = include_str!("../README.md")]

use anyhow::{bail, Context, Result};
use cargo_metadata::MetadataCommand;
use clap::{crate_version, AppSettings, FromArgMatches, IntoApp};
use once_cell::sync::OnceCell;

use std::{
    env,
    fs::{self, read, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

use crate::opt::Opt;

mod opt;

static TARGET_PATH: OnceCell<PathBuf> = OnceCell::new();

fn main() -> Result<()> {
    let target_dir = TARGET_PATH.get_or_init(get_target_dir);
    let args = env::args_os().skip(2);

    let matches = Opt::into_app()
        .version(crate_version!())
        .bin_name("cargo witgen")
        .setting(AppSettings::NoBinaryName)
        .get_matches_from(args);

    let matches =
        Opt::from_arg_matches(&matches).ok_or_else(|| anyhow::anyhow!("Command not found"))?;

    run_generate(target_dir, matches.command)
}

fn run_generate(target_dir: &Path, cli_args: crate::opt::Command) -> Result<()> {
    anyhow::ensure!(
        Path::new("Cargo.toml").exists(),
        r#"Failed to read `Cargo.toml`.
  hint: This command only works in the manifest directory of a Cargo package."#
    );
    let crate::opt::Command::Generate {
        args,
        output,
        prefix_file,
        prefix_string,
    } = cli_args;

    // path to the Cargo executable
    let cargo = env::var("CARGO")
        .context("`generate` subcommand may only be invoked as `cargo witgen generate`")?;

    let check_status = Command::new(&cargo)
        .arg("rustc")
        .args(args)
        .arg("--")
        .arg("--emit")
        .arg("dep-info,metadata")
        // set an always-changing cfg so we can consistently trigger recompile
        .arg("--cfg")
        .arg(format!(
            "__witgen_recompile_trigger=\"{}\"",
            SystemTime::UNIX_EPOCH.elapsed()?.as_millis()
        ))
        .env("WITGEN_ENABLED", "true")
        .status()?;

    if !check_status.success() {
        bail!("`cargo check` failed with status: {}", check_status);
    }

    let pattern = target_dir.join("*.wit");
    let filename = output.unwrap_or_else(|| PathBuf::from("witgen.wit"));

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(filename)
        .expect("cannot create file to generate wit");
    let mut generated_comment = format!("// This is a generated file by witgen (https://github.com/bnjjj/witgen), please do not edit yourself, you can generate a new one thanks to cargo witgen generate command. (cargo-witgen v{}) \n\n", env!("CARGO_PKG_VERSION"));

    if let Some(path) = prefix_file {
        let prefix_file = String::from_utf8(read(path)?)?;
        generated_comment.push_str(&format!("{}\n\n", prefix_file));
    }
    if let Some(prefix) = prefix_string {
        generated_comment.push_str(&format!("{}\n\n", prefix));
    }

    file.write_all(generated_comment.as_bytes())
        .context("cannot write to wit file")?;

    for path in glob::glob(
        pattern
            .to_str()
            .context("CARGO_TARGET_DIR not valid UTF-8")?,
    )? {
        let path = path?;
        let mut content = fs::read(&*path)?;
        content.push(b'\n');

        file.write_all(&content[..])
            .context("cannot write to wit file")?;

        // We don't care too much if we can't remove it
        let _ = fs::remove_file(&path);
    }
    file.flush().context("cannot flush wit file")?;

    Ok(())
}

pub(crate) fn get_target_dir() -> PathBuf {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("cannot fetch cargo metadata");

    metadata.target_directory.join("witgen").into()
}
