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
use crate::framework::Mode;

#[derive(Debug)]
pub struct Step {
    step: Freq,
    state: State,
}

#[derive(Debug)]
enum State {
    Release,
    Limit,
    None,
}

impl Step {
    pub const fn new() -> Self {
        Self {
            step: 0,
            state: State::None,
        }
    }

    pub fn release(&mut self, freq: Freq, target_fps: u32, mode: Mode) -> Freq {
        if matches!(self.state, State::Release) {
            self.step += 25000 * 120 / target_fps as Freq;

            let step_max = match mode {
                Mode::Powersave | Mode::Balance => 75000,
                Mode::Performance | Mode::Fast => 100_000,
            };

            self.step = self.step.min(step_max);
        } else {
            self.step = 25000 * 120 / target_fps as Freq;
            self.state = State::Release;
        }

        freq.saturating_add(self.step)
    }

    pub fn limit(&mut self, freq: Freq, target_fps: u32) -> Freq {
        if !matches!(self.state, State::Limit) {
            self.state = State::Limit;
        }

        self.step = 25000 * 120 / target_fps as Freq;

        freq.saturating_sub(self.step)
    }

    pub fn reset(&mut self) {
        self.state = State::None;
    }
}
