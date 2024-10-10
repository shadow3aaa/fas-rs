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

pub mod api;
mod core;

use std::{
    fs,
    sync::mpsc::{self, SyncSender},
    thread,
};

use crate::framework::error::Result;
pub use api::Api;

const EXTENSIONS_PATH: &str = "/dev/fas_rs/extensions";

pub struct Extension {
    sx: SyncSender<Box<dyn Api>>,
}

impl Extension {
    pub fn init() -> Result<Self> {
        let _ = fs::create_dir_all(EXTENSIONS_PATH);
        let (sx, rx) = mpsc::sync_channel(16);

        thread::Builder::new()
            .name("ExtensionThead".into())
            .spawn(move || core::thread(&rx))?;

        Ok(Self { sx })
    }

    pub fn trigger_extentions(&self, trigger: impl Api + 'static) {
        let _ = self.sx.try_send(trigger.into_box());
    }
}
