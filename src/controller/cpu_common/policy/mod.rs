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

use atomic::Atomic;
use cpu_cycles_reader::Cycles;

use cycles::DiffReader;
use schedule::Schedule;

pub struct Policy {
    target_diff: Arc<Atomic<Cycles>>,
    cur_cycles: Arc<Atomic<Cycles>>,
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
        let (mut schedule, target_diff, cur_cycles) = Schedule::new(policy_path, burst_max);

        let pause = Arc::new(AtomicBool::new(true));
        let exit = Arc::new(AtomicBool::new(false));

        let handle = {
            let pause = pause.clone();
            let exit = exit.clone();
            thread::Builder::new()
                .name("CpuPolicyThread".into())
                .spawn(move || loop {
                    if pause.load(Ordering::Acquire) {
                        schedule.reset();
                        thread::park();
                    } else if exit.load(Ordering::Acquire) {
                        schedule.reset();
                        return;
                    }

                    let cur_freq = schedule.cur_cycles.load(Ordering::Acquire);
                    let diff = reader.read_diff(cur_freq);
                    schedule.run(diff);
                })
                .unwrap()
        };

        Self {
            target_diff,
            cur_cycles,
            pause,
            exit,
            handle,
        }
    }

    pub fn resume(&self) {
        self.pause.store(false, Ordering::Release);
        self.handle.thread().unpark();
    }

    pub fn pause(&self) {
        self.pause.store(true, Ordering::Release);
    }

    // 返回最大可设置cycles
    pub fn set_target_diff(&self, c: Cycles) -> Cycles {
        let c = c.min(self.cur_cycles.load(Ordering::Acquire));
        self.target_diff.store(c, Ordering::Release);
        c
    }
}
