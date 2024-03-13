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
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct Acc {
    len: u32,
    dur: Duration,
}

impl Acc {
    pub const fn new() -> Self {
        Self {
            len: 0,
            dur: Duration::ZERO,
        }
    }

    pub fn timeout_dur(&self) -> Duration {
        self.dur
            .saturating_sub(Duration::from_secs(self.len.into()))
            / self.len
    }

    pub fn acc(&mut self, normalized_frame: Duration) {
        self.len += 1;
        self.dur += normalized_frame;
    }

    pub fn reset(&mut self) {
        self.dur = Duration::ZERO;
        self.len = 0;
    }
}
