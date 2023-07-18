use std::{
    cmp::{self, Ordering as CmpOrdering},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use fas_rs_fw::write_pool::WritePool;

use atomic::{Atomic, Ordering};
use cpu_cycles_reader::Cycles;
use likely_stable::LikelyOption;

use crate::{config::CONFIG, debug};

const BURST_DEFAULT: usize = 0;

pub struct Schedule {
    path: PathBuf,
    target_diff: Arc<Atomic<Cycles>>,
    pub cur_cycles: Arc<Atomic<Cycles>>,
    burst: usize,
    burst_max: usize,
    pool: WritePool,
    table: Vec<Cycles>,
    pos: usize,
}

impl Schedule {
    pub fn new(path: &Path, burst_max: usize) -> (Self, Arc<Atomic<Cycles>>, Arc<Atomic<Cycles>>) {
        let target_diff = Arc::new(Atomic::new(Cycles::from_mhz(200)));
        let target_diff_clone = target_diff.clone();

        let count = fs::read_to_string(path.join("affected_cpus"))
            .unwrap()
            .split_whitespace()
            .count();
        let pool = WritePool::new(cmp::max(count / 2, 2));

        let mut table: Vec<Cycles> = fs::read_to_string(path.join("scaling_available_frequencies"))
            .unwrap()
            .split_whitespace()
            .map(|freq| Cycles::from_khz(freq.parse().unwrap()))
            .collect();

        table.sort_unstable();
        table_spec(&mut table);

        let cur_cycles = Arc::new(Atomic::new(table.last().copied().unwrap()));
        let cur_cycles_clone = cur_cycles.clone();

        debug! {
            println!("{:#?}", &table);
        }

        let pos = table.len() - 1;

        (
            Self {
                path: path.to_owned(),
                target_diff,
                cur_cycles,
                burst: BURST_DEFAULT,
                burst_max,
                pool,
                table: table.clone(),
                pos,
            },
            target_diff_clone,
            cur_cycles_clone,
        )
    }

    pub fn run(&mut self, diff: Cycles) {
        if diff < Cycles::new(0) {
            return;
        }

        let max = self.table[self.pos];
        self.cur_cycles.store(max, Ordering::Release);

        let target_diff = self.target_diff.load(Ordering::Acquire);
        let target_diff = target_diff.min(self.cur_cycles.load(Ordering::Acquire));

        assert!(
            target_diff.as_hz() >= 0,
            "Target diff should never be less than zero, but got {target_diff}"
        );

        match target_diff.cmp(&diff) {
            CmpOrdering::Less => {
                self.pos = self.pos.saturating_sub(1);
                self.write();
                self.burst = BURST_DEFAULT;
            }
            CmpOrdering::Greater => {
                self.pos = cmp::min(self.pos + 1 + self.burst, self.table.len() - 1);
                self.write();
                self.burst = cmp::min(self.burst_max, self.burst + 1);
            }
            CmpOrdering::Equal => self.burst = BURST_DEFAULT,
        }

        table_spec(&mut self.table);
    }

    pub fn reset(&mut self) {
        let _ = self.pool.write(
            &self.path.join("scaling_max_freq"),
            &self.table.last().unwrap().as_khz().to_string(),
        );
    }

    fn write(&mut self) {
        let _ = self.pool.write(
            &self.path.join("scaling_max_freq"),
            &self.table[self.pos].as_khz().to_string(),
        );
    }
}

fn table_spec(table: &mut Vec<Cycles>) {
    let save_count = CONFIG
        .get_conf("freq_count")
        .and_then_likely(|c| usize::try_from(c.as_integer()?).ok())
        .unwrap();

    let len = table.len();

    if len <= save_count {
        return;
    }

    *table = table
        .iter()
        .copied()
        .filter(|f| *f >= Cycles::from_mhz(500))
        .collect();

    *table = table
        .iter()
        .rev()
        .step_by(len / save_count)
        .rev()
        .copied()
        .collect();
}
