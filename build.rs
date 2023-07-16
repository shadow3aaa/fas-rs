use std::{error::Error, fs, io::Write};

use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub repository: String,
}

#[derive(Deserialize)]
struct TomlData {
    pub package: Package,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=make.sh");

    let toml = fs::read_to_string("Cargo.toml")?;
    let data: TomlData = toml::from_str(&toml)?;

    let package = data.package;
    let id = package.name.replace('-', "_"); // 符合magisk module id要求
    let version_code: usize = package.version.replace('.', "").trim().parse()?; // 转为纯数字版本

    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("build_module/module.prop")?;

    writeln!(file, "id={id}")?;
    writeln!(file, "name={}", package.name)?;
    writeln!(file, "version=v{}", package.version)?;
    writeln!(file, "versionCode={version_code}")?;
    writeln!(
        file,
        "description={} {}",
        package.description, package.repository
    )?;

    Ok(())
}
