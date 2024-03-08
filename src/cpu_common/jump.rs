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
use super::Freq;

#[derive(Debug)]
pub struct JumpStep {
    jump: Freq,
    state: State,
}

#[derive(Debug)]
enum State {
    Release,
    Limit,
    None,
}

impl JumpStep {
    pub const fn new() -> Self {
        Self {
            jump: 0,
            state: State::None,
        }
    }

    pub fn release(&mut self, freq: Freq) -> Freq {
        if matches!(self.state, State::Release) {
            self.jump = (self.jump + 5000).min(250_000);
        } else {
            self.jump = 5000;
            self.state = State::Release;
        }

        freq.saturating_add(self.jump)
    }

    pub fn limit(&mut self, freq: Freq) -> Freq {
        if matches!(self.state, State::Limit) {
            self.jump = (self.jump + 5000).min(250_000);
        } else {
            self.jump = 5000;
            self.state = State::Limit;
        }

        freq.saturating_sub(self.jump)
    }

    pub fn reset(&mut self) {
        self.jump = 0;
        self.state = State::None;
    }
}
