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
    positive: Duration,
    negative: Duration,
}

impl Acc {
    pub const fn new() -> Self {
        Self {
            positive: Duration::ZERO,
            negative: Duration::ZERO,
        }
    }

    pub const fn as_duration(&self) -> Duration {
        self.positive.saturating_sub(self.negative)
    }

    pub fn acc(&mut self, normalized_frame: Duration) {
        if normalized_frame > Duration::from_secs(1) {
            self.positive += normalized_frame - Duration::from_secs(1);
        } else {
            self.negative += Duration::from_secs(1) - normalized_frame;
        }
    }

    pub fn reset(&mut self) {
        self.positive = Duration::ZERO;
        self.negative = Duration::ZERO;
    }
}
