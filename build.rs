// Copyright 2023 shadow3aaa@gitbub.com
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{fs, io::Write};

use anyhow::Result;
use serde::{Deserialize, Serialize};

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

enum TracingToolType {
    Ebpf,
    Zygisk,
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

    println!("cargo:rustc-link-search=prebuilt");
    println!("cargo:rustc-link-lib=binder_ndk");

    let toml = fs::read_to_string("Cargo.toml")?;
    let data: CargoConfig = toml::from_str(&toml)?;

    gen_module_prop(&data, TracingToolType::Ebpf)?;
    gen_module_prop(&data, TracingToolType::Zygisk)?;
    update_json(&data, TracingToolType::Ebpf)?;
    update_json(&data, TracingToolType::Zygisk)?;

    fs::copy("update/update_zygisk.json", "update/update.json")?;
    fs::copy("update/update_zygisk_en.json", "update/update_en.json")?;

    Ok(())
}

fn gen_module_prop(data: &CargoConfig, tool_type: TracingToolType) -> Result<()> {
    let package = &data.package;
    let id = package.name.replace('-', "_");
    let version_code: usize = package.version.replace('.', "").trim().parse()?;
    let authors = &package.authors;
    let mut author = String::new();
    for a in authors {
        author += &format!("{a} ");
    }
    let author = author.trim();

    let (prop, tool_type, description_ext) = match tool_type {
        TracingToolType::Ebpf => (
            "module/fas-rs-ebpf/module.prop",
            "Ebpf",
            "Requires kernel ebpf support.",
        ),
        TracingToolType::Zygisk => (
            "module/fas-rs-zygisk/module.prop",
            "Zygisk",
            "Requires Magisk 24.0+ and Zygisk enabled.",
        ),
    };

    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(prop)?;

    writeln!(file, "id={id}")?;
    writeln!(file, "name={tool_type} - {}", package.name)?;
    writeln!(file, "version=v{}", package.version)?;
    writeln!(file, "versionCode={version_code}")?;
    writeln!(file, "author={author}")?;
    writeln!(
        file,
        "description={} {description_ext}",
        package.description
    )?;

    Ok(())
}

fn update_json(data: &CargoConfig, tool_type: TracingToolType) -> Result<()> {
    let version = &data.package.version;
    let version_code: usize = version.replace('.', "").trim().parse()?;
    let version = format!("v{version}");

    let zip_url = match tool_type {
        TracingToolType::Ebpf => format!(
            "https://github.com/shadow3aaa/fas-rs/releases/download/{version}/fas-rs-ebpf.zip"
        ),
        TracingToolType::Zygisk => format!(
            "https://github.com/shadow3aaa/fas-rs/releases/download/{version}/fas-rs-zygisk.zip"
        ),
    };

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

    match tool_type {
        TracingToolType::Ebpf => {
            fs::write("update/update_ebpf.json", cn)?;
            fs::write("update/update_ebpf_en.json", en)?;
        }
        TracingToolType::Zygisk => {
            fs::write("update/update_zygisk.json", cn)?;
            fs::write("update/update_zygisk_en.json", en)?;
        }
    }

    Ok(())
}
