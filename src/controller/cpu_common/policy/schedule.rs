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
};

use crate::controller::write_pool::WritePool;

pub(super) fn schedule_thread(
    path: PathBuf,
    usage: Receiver<u8>,
    target_usage: Arc<AtomicU8>,
    pause: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
) {
    let count = fs::read_to_string(path.join("affect_cpus"))
        .unwrap()
        .split_whitespace()
        .count();
    let mut pool = WritePool::new(count);

    let mut table: Vec<usize> = fs::read_to_string(path.join("scaling_available_frequencies"))
        .unwrap()
        .split_whitespace()
        .map(|freq| freq.parse().unwrap())
        .collect();
    table.sort_unstable();
    let mut pos = table.len() - 1;

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

        let usage = usage.recv().unwrap();
        let target_usage = target_usage.load(AtomOrdering::Acquire);

        match usage.cmp(&target_usage) {
            CmpOrdering::Greater => {
                pos = cmp::min(pos + 1, table.len() - 1);
                pool.write(&path.join("scaling_max_freq"), &table[pos].to_string())
                    .unwrap();
            }
            CmpOrdering::Less => {
                pos = pos.saturating_sub(1);
                pool.write(&path.join("scaling_max_freq"), &table[pos].to_string())
                    .unwrap();
            }
            CmpOrdering::Equal => (),
        }
    }
}
