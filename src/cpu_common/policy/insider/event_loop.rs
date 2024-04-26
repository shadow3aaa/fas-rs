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
    collections::HashMap,
    time::{Duration, Instant},
};

use cpu_cycles_reader::{Cycles, CyclesInstant, CyclesReader};

use super::{Event, Insider};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Fas,
    Normal,
}

impl Insider {
    pub fn event_loop(mut self) {
        let reader = CyclesReader::new().unwrap();
        let mut cycles: HashMap<i32, CyclesInstant> = HashMap::with_capacity(self.cpus.len());
        let mut last_record = Instant::now();

        loop {
            if self.always_userspace_governor() {
                let max_cycles = self.max_cycles(&reader, &mut last_record, &mut cycles);
                self.normal_policy(max_cycles);
            }

            if let Some(event) = self.recv_event() {
                let _ = match event {
                    Event::InitDefault(b) => self.init_default(b),
                    Event::InitGame => self.init_game(),
                    Event::SetFasFreq(f) => self.set_fas_freq(f),
                };
            }
        }
    }

    fn recv_event(&self) -> Option<Event> {
        if self.always_userspace_governor() {
            self.rx.recv_timeout(Duration::from_millis(25)).ok()
        } else {
            self.rx.recv().ok()
        }
    }

    fn max_cycles(
        &self,
        reader: &CyclesReader,
        last_record: &mut Instant,
        map: &mut HashMap<i32, CyclesInstant>,
    ) -> Cycles {
        let mut cycles = Cycles::ZERO;
        for cpu in self.cpus.iter().copied() {
            let now = reader.instant(cpu).unwrap();
            let prev = map.entry(cpu).or_insert(now);

            cycles = cycles.max(now - *prev);
            *prev = now;
        }

        let time = last_record.elapsed();
        *last_record = Instant::now();

        cycles * Duration::from_secs(1).as_nanos() as i64 / time.as_nanos() as i64
    }
}
