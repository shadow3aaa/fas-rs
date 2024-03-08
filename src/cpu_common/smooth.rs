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
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use super::Freq;

const RECORD_LENGTH: Duration = Duration::from_millis(200);

#[derive(Debug)]
pub struct Smooth {
    last_freq: Freq,
    freqs: VecDeque<(Freq, Instant)>,
}

impl Smooth {
    pub const fn new() -> Self {
        Self {
            last_freq: 0,
            freqs: VecDeque::new(),
        }
    }

    pub fn update(&mut self, freq: Freq) {
        self.freqs.push_front((self.last_freq, Instant::now()));
        self.last_freq = freq;
        self.retain();
    }

    fn retain(&mut self) {
        self.freqs.retain_mut(|(_, instant)| {
            let elapsed = instant.elapsed();
            if elapsed >= RECORD_LENGTH * 2 {
                return false;
            }

            if elapsed > RECORD_LENGTH {
                *instant += elapsed - RECORD_LENGTH;
            }

            true
        });
    }

    pub fn avg(&self) -> Option<Freq> {
        let (mut freq, mut start) = self.freqs.back().copied()?;
        let (_, end) = self.freqs.front().copied()?;
        let total = end - start;
        let mut avg = 0;

        for (next_freq, end) in self.freqs.iter().copied().skip(1) {
            let duration = end - start;

            avg += (freq * duration.as_nanos() as usize).checked_div(total.as_nanos() as usize)?;

            freq = next_freq;
            start = end;
        }

        Some(avg)
    }

    pub fn reset(&mut self) {
        self.freqs.clear();
    }
}
