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
use std::collections::VecDeque;

use crate::cpu_common::Freq;

const LENGTH: usize = 1000;

#[derive(Debug)]
pub struct Smooth {
    buffer: VecDeque<Freq>,
}

impl Smooth {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(LENGTH),
        }
    }

    pub fn update(&mut self, f: Freq) -> Freq {
        if self.buffer.len() == LENGTH {
            self.buffer.pop_back();
        }

        self.buffer.push_front(f);
        self.buffer.iter().copied().sum::<Freq>() / LENGTH
    }
}
