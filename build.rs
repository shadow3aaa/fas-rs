// Copyright 2023-2025, shadow3 (@shadow3aaa)
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

use std::{fs, io::Write};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use vergen::{BuildBuilder, CargoBuilder, Emitter, RustcBuilder, SysinfoBuilder};

#[derive(Deserialize)]
struct Package {
    pub authors: Vec<String>,
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Deserialize)]
struct CargoConfig {
    pub package: Package,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct UpdateJson {
    versionCode: usize,
    version: String,
    zipUrl: String,
    changelog: String,
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=update");

    vergen()?;

    let toml = fs::read_to_string("Cargo.toml")?;
    let data: CargoConfig = toml::from_str(&toml)?;

    gen_module_prop(&data)?;
    update_json(&data)?;

    Ok(())
}

fn gen_module_prop(data: &CargoConfig) -> Result<()> {
    let package = &data.package;
    let id = package.name.replace('-', "_");
    let version_code: usize = package.version.replace('.', "").trim().parse()?;
    let authors = &package.authors;
    let mut author = String::new();
    for a in authors {
        author += &format!("{a} ");
    }
    let author = author.trim();

    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("module/module.prop")?;

    writeln!(file, "id={id}")?;
    writeln!(file, "name={}", package.name)?;
    writeln!(file, "version=v{}", package.version)?;
    writeln!(file, "versionCode={version_code}")?;
    writeln!(file, "author={author}")?;
    writeln!(file, "description={}", package.description)?;

    Ok(())
}

fn update_json(data: &CargoConfig) -> Result<()> {
    let version = &data.package.version;
    let version_code: usize = version.replace('.', "").trim().parse()?;
    let version = format!("v{version}");

    let zip_url =
        format!("https://github.com/shadow3aaa/fas-rs/releases/download/{version}/fas-rs.zip");

    let cn = UpdateJson {
        versionCode: version_code,
        version: version.clone(),
        zipUrl: zip_url.clone(),
        changelog: "https://github.com/shadow3aaa/fas-rs/raw/master/update/zh-CN/changelog.md"
            .into(),
    };

    let en = UpdateJson {
        versionCode: version_code,
        version,
        zipUrl: zip_url,
        changelog: "https://github.com/shadow3aaa/fas-rs/raw/master/update/en-US/changelog.md"
            .into(),
    };

    let cn = serde_json::to_string_pretty(&cn)?;
    let en = serde_json::to_string_pretty(&en)?;

    fs::write("update/update.json", cn)?;
    fs::write("update/update_en.json", en)?;

    Ok(())
}

fn vergen() -> Result<()> {
    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;
    let rustc = RustcBuilder::all_rustc()?;
    let si = SysinfoBuilder::all_sysinfo()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&rustc)?
        .add_instructions(&si)?
        .emit()
}
