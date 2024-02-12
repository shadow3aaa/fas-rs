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
mod event_loop;
mod misc;
mod normal;
mod sma;
mod utils;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
    thread,
};

use anyhow::Result;

use super::super::Freq;
use event_loop::State;
use sma::Smooth;

pub enum Event {
    InitDefault(bool),
    InitGame(bool),
    SetFasFreq(Freq),
    SetFasGovernor(bool),
}

#[derive(Debug)]
pub struct Insider {
    cpus: Vec<i32>,
    path: PathBuf,
    cache: Freq,
    smooth: Smooth,
    fas_freq: Freq,
    governor_freq: Freq,
    freqs: Vec<Freq>,
    fas_boost: bool,
    userspace_governor: bool,
    use_performance_governor: bool,
    state: State,
    gov_snapshot: Option<String>,
    rx: Receiver<Event>,
}

impl Insider {
    pub fn spawn<P: AsRef<Path>>(rx: Receiver<Event>, p: P) -> Result<Vec<Freq>> {
        let path = p.as_ref();

        let mut freqs: Vec<Freq> = fs::read_to_string(path.join("scaling_available_frequencies"))?
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();

        if let Ok(boost_freqs) = fs::read_to_string(path.join("scaling_boost_frequencies")) {
            let boost_freqs = boost_freqs
                .split_whitespace()
                .map(|s| s.parse::<Freq>().unwrap());
            freqs.extend(boost_freqs);
        } // 部分设备一部分频率表在scaling_boost_frequencies内

        freqs.sort_unstable();

        let cpus = fs::read_to_string(path.join("affected_cpus"))?;
        let mut cpus: Vec<i32> = cpus
            .split_whitespace()
            .map(|c| c.parse().unwrap())
            .collect();
        cpus.sort_unstable();

        let thread_name = format!("policy {}-{}", cpus[0], cpus.last().unwrap());
        let policy = Self {
            cpus,
            path: path.to_path_buf(),
            freqs: freqs.clone(),
            cache: freqs.last().copied().unwrap(),
            smooth: Smooth::new(),
            fas_freq: freqs.last().copied().unwrap(),
            governor_freq: freqs.last().copied().unwrap(),
            fas_boost: false,
            userspace_governor: false,
            state: State::Normal,
            use_performance_governor: false,
            gov_snapshot: None,
            rx,
        };

        thread::Builder::new()
            .name(thread_name)
            .spawn(move || Self::event_loop(policy))?;

        Ok(freqs)
    }
}
