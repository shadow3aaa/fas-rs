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

use super::super::SetFreqType;
use crate::cpu_common::Freq;

#[derive(Debug)]
pub struct Smooth {
    buffer: VecDeque<(Freq, Instant)>,
}

impl Smooth {
    pub const fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }

    pub fn smooth(&mut self, freq: Freq, t: SetFreqType) -> Option<Freq> {
        match t {
            SetFreqType::Jank | SetFreqType::BigJank => (),
            _ => self.buffer.push_front((freq, Instant::now())),
        }

        self.buffer
            .retain(|(_, i)| i.elapsed() <= Duration::from_secs(30));

        Some(match t {
            SetFreqType::Limit => self.avg()?.max(freq),
            _ => freq,
        })
    }

    fn avg(&self) -> Option<Freq> {
        let sum: Freq = self.buffer.iter().copied().map(|(f, _)| f).sum();
        let len = self.buffer.len();
        sum.checked_div(len)
    }
}
