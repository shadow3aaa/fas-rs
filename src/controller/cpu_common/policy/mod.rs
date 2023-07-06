mod cycles;
mod schedule;

use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use cpu_cycles_reader::Cycles;
use parking_lot::RwLock;

use cycles::DiffReader;
use schedule::Schedule;

pub struct Policy {
    target_diff: Arc<RwLock<Cycles>>,
    pause: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

impl Drop for Policy {
    fn drop(&mut self) {
        self.resume();
        self.exit.store(true, Ordering::Release);
    }
}

impl Policy {
    pub fn new(policy_path: &Path, burst_max: usize) -> Self {
        let mut reader = DiffReader::new(policy_path);
        let (mut schedule, target_diff) = Schedule::new(policy_path, burst_max);

        let pause = Arc::new(AtomicBool::new(false));
        let exit = Arc::new(AtomicBool::new(false));

        let pause_clone = pause.clone();
        let exit_clone = exit.clone();
        let handle = thread::spawn(move || loop {
            if pause_clone.load(Ordering::Acquire) {
                schedule.reset();
                thread::park();
            } else if exit_clone.load(Ordering::Acquire) {
                schedule.reset();
                return;
            }

            let max_freq = schedule.current_freq_max();
            let diff = reader.read_diff(max_freq);
            schedule.run(diff);
        });

        Self {
            target_diff,
            pause,
            exit,
            handle,
        }
    }

    #[inline]
    pub fn resume(&self) {
        self.pause.store(false, Ordering::Release);
        self.handle.thread().unpark();
    }

    #[inline]
    #[allow(unused)]
    pub fn pause(&self) {
        self.pause.store(true, Ordering::Release);
    }

    #[inline]
    pub fn set_target_diff(&self, c: Cycles) {
        *self.target_diff.write() = c;
    }
}
