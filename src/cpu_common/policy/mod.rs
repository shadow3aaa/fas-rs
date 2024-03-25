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
mod insider;

use std::{
    path::Path,
    sync::mpsc::{self, Sender},
};

use anyhow::Result;

use super::Freq;
use crate::framework::prelude::*;
use insider::{Event, Insider};

#[derive(Debug)]
pub struct Policy {
    pub freqs: Vec<Freq>,
    sx: Sender<Event>,
}

impl Policy {
    pub fn new<P: AsRef<Path>>(c: &Config, p: P) -> Result<Self> {
        let (sx, rx) = mpsc::channel();
        let freqs = Insider::spawn(rx, p)?;

        let result = Self { freqs, sx };
        result.init_default(c)?;
        Ok(result)
    }

    pub fn init_default(&self, c: &Config) -> Result<()> {
        let userspace_governor = c.config().userspace_governor;
        self.sx.send(Event::InitDefault(userspace_governor))?;
        Ok(())
    }

    pub fn init_game(&self) -> Result<()> {
        self.sx.send(Event::InitGame)?;
        Ok(())
    }

    pub fn set_fas_freq(&self, f: Freq) -> Result<()> {
        self.sx.send(Event::SetFasFreq(f))?;
        Ok(())
    }
}
