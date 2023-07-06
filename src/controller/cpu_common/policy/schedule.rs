use std::{
    cmp::{self, Ordering as CmpOrdering},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use fas_rs_fw::write_pool::WritePool;

use cpu_cycles_reader::Cycles;
use likely_stable::LikelyOption;
use parking_lot::RwLock;

use crate::config::CONFIG;
use crate::debug;

const BURST_DEFAULT: usize = 0;

pub struct Schedule {
    path: PathBuf,
    target_diff: Arc<RwLock<Cycles>>,
    burst: usize,
    burst_max: usize,
    pool: WritePool,
    pub table: Vec<Cycles>,
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

        let mut table: Vec<Cycles> = fs::read_to_string(path.join("scaling_available_frequencies"))
            .unwrap()
            .split_whitespace()
            .map(|freq| Cycles::from_khz(freq.parse().unwrap()))
            .collect();

        table.sort_unstable();
        table_spec(&mut table);

        debug! {
            println!("{:#?}", &table);
        }

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

    #[inline]
    pub fn current_freq_max(&self) -> Cycles {
        self.table[self.pos]
    }

    pub fn run(&mut self, diff: Cycles) {
        if diff < Cycles::new(0) {
            return;
        }

        table_spec(&mut self.table);

        let target_diff = *self.target_diff.read();
        let target_diff = target_diff.min(self.current_freq_max());

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
