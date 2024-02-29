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
use std::{collections::HashMap, time::Instant};

use dobby_api::Address;

#[derive(Debug, Clone)]
pub struct Info {
    pub pid: i32,
    pub stamps: HashMap<Address, Instant>,
    pub gc_timer: Instant,
}

impl Info {
    pub fn new() -> Self {
        let pid = unsafe { libc::getpid() };

        let stamps = HashMap::new();
        let gc_timer = Instant::now();

        Self {
            pid,
            stamps,
            gc_timer,
        }
    }
}
