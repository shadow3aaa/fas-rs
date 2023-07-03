use std::{
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use cpu_cycles_reader::{Cycles, CyclesReader};
use yata::methods::{DEMA, EMA};
use yata::prelude::*;

use super::reset;
use super::schedule::Schedule;
use crate::config::CONFIG;
use crate::debug;

enum SpecEma {
    Ema(EMA),
    Dema(DEMA),
}

impl SpecEma {
    fn next(&mut self, value: &f64) -> f64 {
        match self {
            Self::Ema(e) => e.next(value),
            Self::Dema(e) => e.next(value),
        }
    }
}

pub(super) fn cycles_thread(
    path: &Path,
    mut schedule: Schedule,
    pause: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
) {
    let affected_cpus: Vec<i32> = fs::read_to_string(path.join("affected_cpus"))
        .unwrap()
        .split_whitespace()
        .map(|cpu| cpu.parse::<i32>().unwrap())
        .collect();

    let window = CONFIG
        .get_conf("EMA_WIN")
        .and_then(|d| d.as_integer())
        .unwrap_or(4) as u8;

    let mut ema = CONFIG
        .get_conf("EMA_TYPE")
        .and_then(|d| match d.as_str()? {
            "EMA" => Some(SpecEma::Ema(EMA::new(window, &0.0).ok()?)),
            "DEMA" => Some(SpecEma::Dema(DEMA::new(window, &0.0).ok()?)),
            _ => None,
        })
        .unwrap();

    let reader = CyclesReader::new(affected_cpus.as_slice()).unwrap();

    reset(path).unwrap();

    loop {
        if exit.load(Ordering::Acquire) {
            reset(path).unwrap();
            return;
        } else if pause.load(Ordering::Acquire) {
            reset(path).unwrap();
            thread::park();
        }

        reader.enable();
        let time = Instant::now();
        let cycles_former = reader.read().unwrap();

        thread::sleep(Duration::from_millis(75));

        let cycles_later = reader.read().unwrap();
        let time = Instant::now() - time;
        reader.disable();

        let cycles = affected_cpus
            .iter()
            .map(|cpu| *cycles_later.get(cpu).unwrap() - *cycles_former.get(cpu).unwrap())
            .max()
            .unwrap();

        let cycles = Cycles::from_khz(ema.next(&(cycles.as_khz() as f64)) as i64);

        let path = path.join("scaling_cur_freq");
        let cur_freq = fs::read_to_string(&path).unwrap();
        let cur_freq_cycles = cur_freq.trim().parse().map(Cycles::from_khz).unwrap();

        let cycles = cycles.as_diff(time, cur_freq_cycles).unwrap();
        schedule.run(cycles, cur_freq_cycles);
        
        debug! {
            println!("diff: {}", cycles);
        }
    }
}
