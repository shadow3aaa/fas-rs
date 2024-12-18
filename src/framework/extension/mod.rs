// Copyright 2023-2024, shadow3 (@shadow3aaa)
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
            .name("ExtensionThread".into())
            .spawn(move || core::thread(&rx))?;

        Ok(Self { sx })
    }

    pub fn trigger_extentions(&self, trigger: impl Api + 'static) {
        let _ = self.sx.try_send(trigger.into_box());
    }
}
