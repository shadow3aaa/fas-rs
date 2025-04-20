// Copyright 2023-2025, shadow3, shadow3aaa
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

#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]

mod cpu_common;
mod file_handler;
mod framework;
mod misc;

use std::{
    env, fs,
    io::{self, prelude::*},
    process,
    process::Command,
};

use framework::prelude::*;

use actix_web::{get, web, App, HttpServer, Responder};
use anyhow::Result;
use flexi_logger::{DeferredNow, LogSpecification, Logger, Record};
use log::{error, warn};
use mimalloc::MiMalloc;
use serde::Serialize;

#[cfg(debug_assertions)]
use log::debug;

use cpu_common::Controller;
use misc::setprop;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const USER_CONFIG: &str = "/sdcard/Android/fas-rs/games.toml";

#[derive(Debug, Serialize)]
struct AppInfo {
    name: String,
    package_name: String,
}

#[derive(Debug)]
struct PackageInfo {
    app_name: String,
    package_name: String,
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();

    if args[1] == "merge" {
        let local = fs::read_to_string(USER_CONFIG)?;
        let std = fs::read_to_string(&args[2])?;

        let new = Config::merge(&local, &std).unwrap_or(std);
        println!("{new}");

        return Ok(());
    } else if args[1] == "run" {
        setprop("fas-rs-server-started", "true");
        
        std::thread::spawn(|| {
            start_webserver().expect("Web server failed");
        });
        
        run(&args[2]).unwrap_or_else(|e| {
            for cause in e.chain() {
                error!("{cause:#?}");
            }
            error!("{:#?}", e.backtrace());
        });
    }

    Ok(())
}

fn start_webserver() -> Result<()> {
    actix_web::rt::System::new().block_on(async {
        HttpServer::new(|| {
            App::new().service(get_installed_apps)
        })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
    }).map_err(Into::into)
}

#[get("/api/apps")]
async fn get_installed_apps() -> impl Responder {
    match get_installed_packages() {
        Ok(packages) => {
            let apps = packages
                .into_iter()
                .map(|pkg| AppInfo {
                    name: pkg.app_name,
                    package_name: pkg.package_name,
                })
                .collect::<Vec<_>>();
            web::Json(apps)
        }
        Err(_) => web::Json(Vec::<AppInfo>::new()),
    }
}

fn get_installed_packages() -> Result<Vec<PackageInfo>> {
    let output = Command::new("pm")
        .args(["list", "packages", "-f"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to get package list"));
    }

    let output_str = String::from_utf8(output.stdout)?;
    let packages = output_str
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let package_part = parts[1].splitn(2, '=').collect::<Vec<&str>>();
                if package_part.len() == 2 {
                    let package_name = package_part[1].trim().to_string();
                    return Some(PackageInfo {
                        app_name: package_name.clone(),
                        package_name,
                    });
                }
            }
            None
        })
        .collect();

    Ok(packages)
}

fn run<S: AsRef<str>>(std_path: S) -> Result<()> {
    #[cfg(not(debug_assertions))]
    let logger_spec = LogSpecification::info();

    #[cfg(debug_assertions)]
    let logger_spec = LogSpecification::debug();

    Logger::with(logger_spec)
        .log_to_stdout()
        .format(log_format)
        .start()?;

    let std_path = std_path.as_ref();

    let self_pid = process::id();
    let _ = fs::write("/dev/cpuset/background/cgroup.procs", self_pid.to_string());

    let config = Config::new(USER_CONFIG, std_path)?;
    let cpu = Controller::new()?;

    #[cfg(debug_assertions)]
    debug!("{cpu:#?}");

    Scheduler::new()
        .config(config)
        .controller(cpu)
        .start_run()?;

    Ok(())
}

fn log_format(
    write: &mut dyn Write,
    now: &mut DeferredNow,
    record: &Record<'_>,
) -> Result<(), io::Error> {
    let time = now.format("%Y-%m-%d %H:%M:%S");
    write!(write, "[{time}] {}: {}", record.level(), record.args())
}