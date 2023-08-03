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
    fs,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use cpu_cycles_reader::{Cycles, CyclesReader};
use likely_stable::LikelyOption;
use log::debug;
use yata::{
    methods::{DEMA, EMA, SMA},
    prelude::*,
};

use fas_rs_fw::config::CONFIG;

enum SpecEma {
    Ema(EMA),
    Dema(DEMA),
    Sma(SMA),
    None,
}

pub struct DiffReader {
    affected_cpus: Vec<i32>,
    ema: SpecEma,
    reader: CyclesReader,
}

impl SpecEma {
    fn next(&mut self, value: f64) -> f64 {
        match self {
            Self::Ema(e) => e.next(&value),
            Self::Dema(e) => e.next(&value),
            Self::Sma(e) => e.next(&value),
            Self::None => value,
        }
    }
}

impl DiffReader {
    pub fn new(path: &Path) -> Self {
        let affected_cpus: Vec<i32> = fs::read_to_string(path.join("affected_cpus"))
            .unwrap()
            .split_whitespace()
            .map(|cpu| cpu.parse::<i32>().unwrap())
            .collect();

        let window = CONFIG
            .get_conf("EMA_WIN")
            .and_then_likely(|d| u8::try_from(d.as_integer()?).ok())
            .unwrap_or(4);

        let ema = CONFIG
            .get_conf("EMA_TYPE")
            .and_then_likely(|d| match d.as_str()? {
                "EMA" => Some(SpecEma::Ema(EMA::new(window, &0.0).ok()?)),
                "DEMA" => Some(SpecEma::Dema(DEMA::new(window, &0.0).ok()?)),
                "SMA" => Some(SpecEma::Sma(SMA::new(window, &0.0).ok()?)),
                "None" => Some(SpecEma::None),
                _ => None,
            })
            .unwrap();

        let reader = CyclesReader::new(affected_cpus.as_slice()).unwrap();
        reader.enable();

        Self {
            affected_cpus,
            ema,
            reader,
        }
    }

    pub fn read_diff(&mut self, cur_freq: Cycles) -> Cycles {
        let time = Instant::now();
        let cycles_former = self.reader.read().unwrap();

        thread::sleep(Duration::from_millis(50));

        let cycles_later = self.reader.read().unwrap();
        let time = time.elapsed();

        let cycles = self
            .affected_cpus
            .iter()
            .map(|cpu| *cycles_later.get(cpu).unwrap() - *cycles_former.get(cpu).unwrap())
            .max()
            .unwrap();

        let diff = cycles.as_diff(time, cur_freq).unwrap().max(0.into());

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        let diff = Cycles::from_hz(self.ema.next(diff.as_hz() as f64).round() as i64);

        debug!("Got diff {diff}");
        diff
    }
}
