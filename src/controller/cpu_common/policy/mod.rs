mod schedule;
mod usage;

use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc, Arc,
    },
    thread::{self, JoinHandle},
};

use schedule::*;
use usage::*;

pub struct Policy {
    target_usage: Arc<AtomicU8>,
    pause: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
    handles: [JoinHandle<()>; 2],
}

impl Drop for Policy {
    fn drop(&mut self) {
        self.resume();
        self.exit.store(true, Ordering::Release);
    }
}

impl Policy {
    pub fn new(policy_path: &Path, burst_max: usize) -> Self {
        let (usage_sx, usage_rx) = mpsc::channel();

        let target_usage = Arc::new(AtomicU8::new(75));
        let pause = Arc::new(AtomicBool::new(false));
        let exit = Arc::new(AtomicBool::new(false));

        let pause_clone = pause.clone();
        let exit_clone = exit.clone();
        let path_clone = policy_path.to_owned();
        let usage_thread =
            thread::spawn(move || usage_thread(path_clone, usage_sx, pause_clone, exit_clone));

        let pause_clone = pause.clone();
        let exit_clone = exit.clone();
        let path_clone = policy_path.to_owned();
        let target_clone = target_usage.clone();
        let schedule_thread = thread::spawn(move || {
            schedule_thread(
                path_clone,
                usage_rx,
                target_clone,
                burst_max,
                pause_clone,
                exit_clone,
            )
        });

        let handles = [usage_thread, schedule_thread];

        Self {
            target_usage,
            pause,
            exit,
            handles,
        }
    }

    pub fn resume(&self) {
        self.pause.store(false, Ordering::Release);
        self.handles
            .iter()
            .for_each(|handle| handle.thread().unpark());
    }

    #[allow(unused)]
    pub fn pause(&self) {
        self.pause.store(true, Ordering::Release);
    }

    pub fn set_target_usage(&self, t: u8) {
        assert!(t <= 100, "target usage should never be greater than 100");
        self.target_usage.store(t, Ordering::Release);
    }
}
