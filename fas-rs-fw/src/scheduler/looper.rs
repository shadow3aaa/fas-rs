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
use std::sync::mpsc::Receiver;

use log::debug;

use super::binder::BinderData;
use crate::{
    config::Config,
    error::{Error, Result},
    PerformanceController,
};

pub struct Looper<P: PerformanceController> {
    rx: Receiver<BinderData>,
    config: Config,
    controller: P,
}

impl<P: PerformanceController> Looper<P> {
    pub fn new(rx: Receiver<BinderData>, config: Config, controller: P) -> Self {
        Self {
            rx,
            config,
            controller,
        }
    }

    pub fn enter_loop(&self) -> Result<()> {
        loop {
            let data = self
                .rx
                .recv()
                .map_err(|_e| Error::Other("Got an error when recving binder data"))?;

            debug!("{data:?}");
        }
    }
}
