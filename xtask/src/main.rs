// Copyright 2025-2025, dependabot[bot], shadow3aaa
//
// This file is part of fas-rs.
//
// fas-rs is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along
// with fas-rs. If not, see <https://www.gnu.org/licenses/>.

mod zip_ext;

use std::{
    fs::{self},
    path::{Path, PathBuf},
    process::{self, Command},
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use fs_extra::{dir, file};
use zip::{CompressionMethod, write::FileOptions};

use zip_ext::zip_create_from_directory_with_options;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check the build of fas-rs
    Check {
        /// Build in release mode (default: false)
        #[clap(short, long, default_value = "false")]
        release: bool,

        /// Print detailed output (default: false)
        #[clap(short, long, default_value = "false")]
        verbose: bool,
    },

    /// Build fas-rs
    Build {
        /// Build in release mode (default: false)
        #[clap(short, long, default_value = "false")]
        release: bool,

        /// Print detailed output (default: false)
        #[clap(short, long, default_value = "false")]
        verbose: bool,
    },

    /// Clean build artifacts
    Clean,

    /// Format source code
    Format {
        /// Print detailed output (default: false)
        #[clap(short, long, default_value = "false")]
        verbose: bool,
    },

    /// Run the Clippy linter
    Lint {
        /// Automatically fix lint issues (default: false)
        #[clap(short, long, default_value = "false")]
        fix: bool,
    },

    /// Update project dependencies
    Update,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let Some(command) = cli.command else {
        eprintln!("No command specified. Use --help to see available commands.");
        process::exit(1);
    };

    match command {
        Commands::Check { release, verbose } => {
            check(release, verbose)?;
        }
        Commands::Build { release, verbose } => {
            build(release, verbose)?;
        }
        Commands::Clean => {
            clean()?;
        }
        Commands::Format { verbose } => {
            format(verbose)?;
        }
        Commands::Lint { fix } => {
            lint(fix)?;
        }
        Commands::Update => {
            update()?;
        }
    }

    Ok(())
}

fn build(release: bool, verbose: bool) -> Result<()> {
    let temp_dir = temp_dir(release);

    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir)?;

    let mut cargo = cargo_ndk();
    cargo.args([
        "build",
        "--target",
        "aarch64-linux-android",
        "-Z",
        "build-std",
        "-Z",
        "trim-paths",
    ]);

    if release {
        cargo.arg("--release");
    }

    if verbose {
        cargo.arg("--verbose");
    }

    cargo.spawn()?.wait()?;

    let module_dir = module_dir();
    dir::copy(
        &module_dir,
        &temp_dir,
        &dir::CopyOptions::new().overwrite(true).content_only(true),
    )
    .unwrap();
    fs::remove_file(temp_dir.join(".gitignore")).unwrap();
    file::copy(
        bin_path(release),
        temp_dir.join("fas-rs"),
        &file::CopyOptions::new().overwrite(true),
    )
    .unwrap();

    build_webui()?;
    dir::copy(
        webroot_dir(),
        &temp_dir,
        &dir::CopyOptions::new().overwrite(true),
    )
    .unwrap();

    let build_type = if release { "release" } else { "debug" };
    let package_path = Path::new("output").join(format!("fas-rs({build_type}).zip"));

    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));
    zip_create_from_directory_with_options(&package_path, &temp_dir, |_| options).unwrap();

    println!("fas-rs built successfully: {:?}", package_path);

    Ok(())
}

fn check(release: bool, verbose: bool) -> Result<()> {
    let mut cargo = cargo_ndk();
    cargo.args([
        "check",
        "--target",
        "aarch64-linux-android",
        "-Z",
        "build-std",
        "-Z",
        "trim-paths",
    ]);
    cargo.env("RUSTFLAGS", "-C default-linker-libraries");

    if release {
        cargo.arg("--release");
    }

    if verbose {
        cargo.arg("--verbose");
    }

    cargo.spawn()?.wait()?;

    Ok(())
}

fn clean() -> Result<()> {
    let temp_dir = temp_dir(false);
    let _ = fs::remove_dir_all(&temp_dir);

    Command::new("cargo").arg("clean").spawn()?.wait()?;

    Ok(())
}

fn format(verbose: bool) -> Result<()> {
    let mut command = Command::new("cargo");
    command.args(["fmt", "--all"]);
    if verbose {
        command.arg("--verbose");
    }
    command.spawn()?.wait()?;

    Ok(())
}

fn lint(fix: bool) -> Result<()> {
    let command_builder = |fix: bool| {
        let mut command = cargo_ndk();
        command.arg("clippy");
        if fix {
            command.args(["--fix", "--allow-dirty", "--allow-staged", "--all"]);
        }
        command.args(["--target", "aarch64-linux-android"]);
        command
    };

    command_builder(fix).spawn()?.wait()?;
    command_builder(fix).arg("--release").spawn()?.wait()?;

    Ok(())
}

fn update() -> Result<()> {
    Command::new("cargo")
        .args(["update", "--recursive"])
        .spawn()?
        .wait()?;
    Command::new("cargo")
        .current_dir("xtask")
        .args(["update", "--recursive"])
        .spawn()?
        .wait()?;

    Ok(())
}

fn module_dir() -> PathBuf {
    Path::new("module").to_path_buf()
}

fn temp_dir(release: bool) -> PathBuf {
    Path::new("output")
        .join(".temp")
        .join(if release { "release" } else { "debug" })
}

fn bin_path(release: bool) -> PathBuf {
    Path::new("target")
        .join("aarch64-linux-android")
        .join(if release { "release" } else { "debug" })
        .join("fas-rs")
}

fn cargo_ndk() -> Command {
    let mut command = Command::new("cargo");
    command
        .args(["+nightly", "ndk", "--platform", "31", "-t", "arm64-v8a"])
        .env("RUSTFLAGS", "-C default-linker-libraries");
    command
}

fn webroot_dir() -> PathBuf {
    Path::new("webui").join("webroot")
}

fn build_webui() -> Result<()> {
    let npm = || {
        let mut command = Command::new("npm");
        command.current_dir("webui");
        command
    };

    npm().arg("install").spawn()?.wait()?;
    npm().args(["run", "build"]).spawn()?.wait()?;

    Ok(())
}
