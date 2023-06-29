use std::{
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use yata::methods::DEMA;
use yata::prelude::*;

use super::reset;
use super::schedule::Schedule;
use crate::config::CONFIG;
use crate::debug;

pub(super) fn usage_thread(
    path: &Path,
    mut schedule: Schedule,
    pause: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
) {
    let affected_cpus: Vec<String> = fs::read_to_string(path.join("affected_cpus"))
        .unwrap()
        .split_whitespace()
        .map(|cpu| format!("cpu{}", cpu))
        .collect();

    let window = CONFIG
        .get_conf("DEMA")
        .and_then(|d| d.as_integer())
        .unwrap_or(4);
    let mut dema = DEMA::new(window as u8, &0.0).unwrap(); // 指数平滑

    reset(path).unwrap();

    loop {
        if exit.load(Ordering::Acquire) {
            reset(path).unwrap();
            return;
        } else if pause.load(Ordering::Acquire) {
            reset(path).unwrap();
            thread::park();
        }

        let stat_a = read_stat(&affected_cpus);
        thread::sleep(Duration::from_millis(50));
        let stat_b = read_stat(&affected_cpus);

        let new_usage = dema.next(
            &(stat_a
                .iter()
                .zip(stat_b.iter())
                .map(|((total_a, idle_a), (total_b, idle_b))| {
                    let total = (total_b - total_a) as f64;
                    let idle = (idle_b - idle_a) as f64;
                    100.0 - idle * 100.0 / total
                })
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()),
        );
        let new_usage = new_usage.min(100.0).max(0.0);
        debug! { println!("{:.2}%", new_usage) }
        schedule.run(new_usage);
    }
}

fn read_stat(affected_cpus: &[String]) -> Vec<(usize, usize)> {
    fs::read_to_string("/proc/stat")
        .unwrap()
        .lines()
        .filter(|line| affected_cpus.iter().any(|cpu| line.starts_with(cpu)))
        .map(|cpu| {
            (
                cpu.split_whitespace()
                    .skip(1)
                    .map(|time| time.parse::<usize>().unwrap())
                    .sum::<usize>(),
                cpu.split_whitespace()
                    .nth(4)
                    .unwrap()
                    .parse::<usize>()
                    .unwrap(),
            )
        })
        .collect()
}
