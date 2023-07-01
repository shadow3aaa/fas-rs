use std::{
    cmp::{self, Ordering as CmpOrdering},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::config::CONFIG;
use crate::debug;
use cpu_cycles_reader::Cycles;
use fas_rs_fw::write_pool::WritePool;
use parking_lot::RwLock;

const BURST_DEFAULT: usize = 0;

pub struct Schedule {
    path: PathBuf,
    target_diff: Arc<RwLock<Cycles>>,
    burst: usize,
    burst_max: usize,
    pool: WritePool,
    table: Vec<usize>,
    pos: usize,
}

impl Schedule {
    pub fn new(path: &Path, burst_max: usize) -> (Self, Arc<RwLock<Cycles>>) {
        let target_diff = Arc::new(RwLock::new(Cycles::from_mhz(200)));

        let target_diff_clone = target_diff.clone();

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
                target_diff,
                burst: BURST_DEFAULT,
                burst_max,
                pool,
                table,
                pos,
            },
            target_diff_clone,
        )
    }

    pub fn run(&mut self, diff: Cycles) {
        let target_diff = *self.target_diff.read();

        match target_diff.cmp(&diff) {
            CmpOrdering::Less => {
                self.pos = self.pos.saturating_sub(1);
                self.pool
                    .write(
                        &self.path.join("scaling_max_freq"),
                        &self.table[self.pos].to_string(),
                    )
                    .unwrap();
                self.burst = BURST_DEFAULT;
            }
            CmpOrdering::Greater => {
                self.pos = cmp::min(self.pos + 1 + self.burst, self.table.len() - 1);
                self.pool
                    .write(
                        &self.path.join("scaling_max_freq"),
                        &self.table[self.pos].to_string(),
                    )
                    .unwrap();
                self.burst = cmp::min(self.burst_max, self.burst + 1);
            }
            CmpOrdering::Equal => self.burst = BURST_DEFAULT,
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
        .rev()
        .collect()
}
