use std::{
    cmp::{self, Ordering as CmpOrdering},
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering as AtomOrdering},
        mpsc::Receiver,
        Arc,
    },
    thread,
    time::Duration,
};

use crate::controller::write_pool::WritePool;

pub(super) fn schedule_thread(
    path: PathBuf,
    usage: Receiver<u8>,
    target_usage: Arc<AtomicU8>,
    burst_max: usize,
    pause: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
) {
    let count = fs::read_to_string(path.join("affected_cpus"))
        .unwrap()
        .split_whitespace()
        .count();
    let mut pool = WritePool::new(cmp::max(count / 2, 2));

    let mut table: Vec<usize> = fs::read_to_string(path.join("scaling_available_frequencies"))
        .unwrap()
        .split_whitespace()
        .map(|freq| freq.parse().unwrap())
        .collect();
    table.sort_unstable();
    let mut pos = table.len() - 1;
    let mut burst = 0;

    pool.write(
        &path.join("scaling_max_freq"),
        &table.iter().max().unwrap().to_string(),
    )
    .unwrap();

    thread::park();

    loop {
        if exit.load(AtomOrdering::Acquire) {
            return;
        } else if pause.load(AtomOrdering::Acquire) {
            pool.write(
                &path.join("scaling_max_freq"),
                &table.iter().max().unwrap().to_string(),
            )
            .unwrap();
            thread::park();
            let _ = usage.iter().count(); // 清空
        }

        let usage = match usage.try_recv() {
            Ok(o) => o,
            Err(_) => {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
        };
        let target_usage = target_usage.load(AtomOrdering::Acquire);

        match usage.cmp(&target_usage) {
            CmpOrdering::Greater => {
                pos = cmp::min(pos + 1 + burst, table.len() - 1);
                pool.write(&path.join("scaling_max_freq"), &table[pos].to_string())
                    .unwrap();
                burst = cmp::min(burst_max, burst + 1);
            }
            CmpOrdering::Less => {
                pos = pos.saturating_sub(1);
                pool.write(&path.join("scaling_max_freq"), &table[pos].to_string())
                    .unwrap();
                burst = 0;
            }
            CmpOrdering::Equal => burst = 0,
        }
    }
}
