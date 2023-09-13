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
mod IRemoteService;

use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use binder::{BinderFeatures, Interface};
use log::{error, info};
use parking_lot::Mutex;

use super::FasData;
use crate::{
    config::Config,
    error::{Error, Result},
};
use IRemoteService::BnRemoteService;

pub struct FasServer {
    config: Config,
    sx: Mutex<Sender<FasData>>,
}

impl Interface for FasServer {}

impl IRemoteService::IRemoteService for FasServer {
    #[allow(clippy::cast_sign_loss)]
    fn sendFrameData(&self, pkg: &str, frametime_ns: i64) -> binder::Result<bool> {
        let Some(target_fps) = self.config.target_fps(pkg) else {
            return Ok(false);
        };

        let frametime = Duration::from_nanos(frametime_ns as u64);

        let data = FasData {
            target_fps,
            pkg: pkg.to_string(),
            frametime,
        };

        if let Err(e) = self.sx.lock().send(data) {
            error!("{e:?}");
        }

        Ok(true)
    }
}

impl FasServer {
    pub fn run_server(config: Config) -> Result<Receiver<FasData>> {
        let (sx, rx) = mpsc::channel();
        let server = Self {
            config,
            sx: Mutex::new(sx),
        };
        let server = BnRemoteService::new_binder(server, BinderFeatures::default());

        binder::add_service("fas_rs_server", server.as_binder())
            .map_err(|_| Error::Other("Failed to register binder service?"))?;

        info!("Binder server started");

        Ok(rx)
    }
}
