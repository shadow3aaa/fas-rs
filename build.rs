/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
use std::{fs, io::Write};

use anyhow::Result;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
struct Package {
    pub authors: Vec<String>,
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Deserialize)]
struct TomlData {
    pub package: Package,
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=Cargo.toml");

    println!("cargo:rustc-link-search=prebuilt");
    println!("cargo:rustc-link-lib=binder_ndk");

    let toml = fs::read_to_string("Cargo.toml")?;
    let data: TomlData = toml::from_str(&toml)?;

    let package = data.package;
    let id = package.name.replace('-', "_"); // 符合magisk module id要求
    let version_code: usize = package.version.replace('.', "").trim().parse()?; // 转为纯数字版本
    let authors = package.authors;
    let mut author = String::new();
    for a in authors {
        author = format!("{author}{a} ");
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
    writeln!(
        file,
        "updateJson=https://github.com/shadow3aaa/fas-rs/raw/master/update/update.json"
    )?;

    Ok(())
}
