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
#![allow(non_snake_case)]
mod IRemoteService;

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use anyhow::Result;
use binder::{BinderFeatures, Interface};
use log::info;

use IRemoteService::BnRemoteService;

pub struct UperfExtension {
    fas_status: Arc<AtomicBool>,
}

impl UperfExtension {
    pub fn run_server(fas_status: Arc<AtomicBool>) -> Result<()> {
        thread::Builder::new()
            .name("BinderServer".into())
            .spawn(|| Self::run(fas_status))?;

        Ok(())
    }

    fn run(fas_status: Arc<AtomicBool>) -> Result<()> {
        let server = Self { fas_status };
        let server = BnRemoteService::new_binder(server, BinderFeatures::default());

        binder::add_service("fas_rs_server_uperf", server.as_binder())?;

        info!("Binder server started");
        binder::ProcessState::join_thread_pool();

        Ok(())
    }
}

impl Interface for UperfExtension {}

impl IRemoteService::IRemoteService for UperfExtension {
    fn writeFreq(&self, freq: i64, path: &str) -> binder::Result<()> {
        if !self.fas_status.load(Ordering::Acquire) {
            let _ = fs::set_permissions(path, PermissionsExt::from_mode(0o644));
            let _ = fs::write(path, freq.to_string());
            let _ = fs::set_permissions(path, PermissionsExt::from_mode(0o444));
        }

        Ok(())
    }
}
