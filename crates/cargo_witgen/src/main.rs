use anyhow::{bail, Context, Result};
use cargo_metadata::MetadataCommand;
use clap::{App, SubCommand};
use once_cell::sync::OnceCell;

use std::{
    env::{self, args},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{self, Command},
    time::SystemTime,
};

static TARGET_PATH: OnceCell<PathBuf> = OnceCell::new();

fn main() -> Result<()> {
    let target_dir = TARGET_PATH.get_or_init(get_target_dir);
    let args = env::args_os().skip(1);

    let matches = App::new("witgen")
        .about("CLI to generate wit files thanks to wit_generator crate")
        .subcommand(
            // App::new("witgen")
            // .subcommand(
            SubCommand::with_name("generate").about("generate wit files"), // ),
        )
        .get_matches_from(args);

    match matches.subcommand() {
        ("generate", Some(_args)) => {
            run_generate(target_dir)?;
        }
        _ => {
            eprintln!("Command not found");
            process::exit(1);
        }
    }

    Ok(())
}

fn run_generate(target_dir: &Path) -> Result<()> {
    anyhow::ensure!(
        Path::new("Cargo.toml").exists(),
        r#"Failed to read `Cargo.toml`.
hint: This command only works in the manifest directory of a Cargo package."#
    );

    // path to the Cargo executable
    let cargo = env::var("CARGO")
        .context("`generate` subcommand may only be invoked as `cargo witgen generate`")?;

    let check_status = Command::new(&cargo)
        .arg("rustc")
        .args(args().skip(3))
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

    // TODO: find better filename
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("witgen.wit")
        .expect("cannot create file to generate wit");

    for path in glob::glob(
        pattern
            .to_str()
            .context("CARGO_TARGET_DIR not valid UTF-8")?,
    )? {
        let path = path?;
        let mut content = fs::read(&*path)?;
        content.push(b'\n');

        file.write_all(&content[..])
            .expect("cannot write to wit file");

        // lazily remove the file, we don't care too much if we can't
        let _ = fs::remove_file(&path);
    }
    file.flush().expect("cannot flush wit file");

    Ok(())
}

pub(crate) fn get_target_dir() -> PathBuf {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("cannot fetch cargo metadata");

    metadata.target_directory.join("witgen").into()
}
