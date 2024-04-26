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

use std::{
    sync::mpsc::{self, Receiver, SyncSender},
    time::Instant,
};

use once_cell::sync::Lazy;

pub static CHANNEL: Lazy<Channel> = Lazy::new(|| {
    let (sx, rx) = mpsc::sync_channel(1024);
    Channel { sx, rx }
});

pub struct Channel {
    pub sx: SyncSender<Instant>,
    pub rx: Receiver<Instant>,
}

unsafe impl Sync for Channel {}
unsafe impl Send for Channel {}
