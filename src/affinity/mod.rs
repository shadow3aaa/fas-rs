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

mod applyer;
mod helper_thread;

use std::{
    sync::mpsc::{self, Sender},
    thread::{self},
};

use helper_thread::{affinity_helper, Command};

pub struct Affinity {
    sx: Sender<Command>,
}

impl Affinity {
    pub fn new() -> Self {
        let (sx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("AffinityHelper".into())
            .spawn(move || affinity_helper(&rx))
            .unwrap();
        Self { sx }
    }

    pub fn attach(&self, pid: i32) {
        let _ = self.sx.send(Command::Attach(pid));
    }

    pub fn detach(&self) {
        let _ = self.sx.send(Command::Detach);
    }
}
