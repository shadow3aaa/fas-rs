use std::{
    cmp, fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU8, Ordering as AtomOrdering},
        Arc,
    },
};

use crate::config::CONFIG;
use crate::debug;
use fas_rs_fw::write_pool::WritePool;

const BURST_DEFAULT: usize = 1;

pub struct Schedule {
    path: PathBuf,
    target_usage: Arc<[AtomicU8; 2]>,
    burst: usize,
    burst_max: usize,
    pool: WritePool,
    table: Vec<usize>,
    pos: usize,
}

impl Schedule {
    pub fn new(path: &Path, burst_max: usize) -> (Self, Arc<[AtomicU8; 2]>) {
        let target_usage = Arc::new([AtomicU8::new(65), AtomicU8::new(68)]);
        let target_usage_clone = target_usage.clone();

        let count = fs::read_to_string(path.join("affected_cpus"))
            .unwrap()
            .split_whitespace()
            .count();
        let pool = WritePool::new(cmp::max(count / 2, 2));

        let table: Vec<usize> = fs::read_to_string(path.join("scaling_available_frequencies"))
            .unwrap()
            .split_whitespace()
            .map(|freq| freq.parse().unwrap())
            .collect();

        let keep_count = CONFIG
            .get_conf("freq_count")
            .and_then(|c| c.as_integer())
            .unwrap_or(8);
        let table = table_spec(table, keep_count as usize);
        debug! { println!("{:#?}", &table) }

        let pos = table.len() - 1;

        (
            Self {
                path: path.to_owned(),
                target_usage,
                burst: BURST_DEFAULT,
                burst_max,
                pool,
                table,
                pos,
            },
            target_usage_clone,
        )
    }

    pub fn run(&mut self, usage: f64) {
        let target_usage = [
            self.target_usage[0].load(AtomOrdering::Acquire),
            self.target_usage[1].load(AtomOrdering::Acquire),
        ];

        if usage < target_usage[0].into() {
            self.pos = self.pos.saturating_sub(1);
            self.pool
                .write(
                    &self.path.join("scaling_max_freq"),
                    &self.table[self.pos].to_string(),
                )
                .unwrap();
            self.burst = BURST_DEFAULT;
        } else if usage > target_usage[1].into() {
            self.pos = cmp::min(self.pos + 1 + self.burst, self.table.len() - 1);
            self.pool
                .write(
                    &self.path.join("scaling_max_freq"),
                    &self.table[self.pos].to_string(),
                )
                .unwrap();
            self.burst = cmp::min(self.burst_max, self.burst + 1);
        } else {
            self.burst = BURST_DEFAULT;
        }
    }
}

fn table_spec(mut table: Vec<usize>, save_count: usize) -> Vec<usize> {
    table.sort_unstable();

    let len = table.len();
    if len <= save_count {
        return table;
    }

    /* let split_freq = table.last().unwrap() / 4;
    table = table
        .iter()
        .filter(|f| **f >= split_freq)
        .copied()
        .collect(); */

    table
        .into_iter()
        .rev()
        .step_by(len / save_count)
        .take(save_count)
        .rev()
        .collect()
}
