mod schedule;
mod usage;

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crate::debug;
use schedule::Schedule;
use usage::*;

pub struct Policy {
    path: PathBuf,
    target_usage: Arc<[AtomicU8; 2]>,
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
        let (schedule, target_usage) = Schedule::new(policy_path, burst_max);

        let pause = Arc::new(AtomicBool::new(false));
        let exit = Arc::new(AtomicBool::new(false));

        let pause_clone = pause.clone();
        let exit_clone = exit.clone();
        let path_clone = policy_path.to_owned();
        let handle =
            thread::spawn(move || usage_thread(&path_clone, schedule, pause_clone, exit_clone));

        Self {
            path: policy_path.to_owned(),
            target_usage,
            pause,
            exit,
            handle,
        }
    }

    pub fn resume(&self) {
        self.pause.store(false, Ordering::Release);
        self.handle.thread().unpark();
    }

    #[allow(unused)]
    pub fn pause(&self) {
        self.pause.store(true, Ordering::Release);
        reset(&self.path).unwrap();
    }

    pub fn set_target_usage(&self, l: u8, r: u8) {
        assert!(r <= 100, "target usage should never be greater than 100");
        assert!(l <= r, "Invalid closed range");

        self.target_usage[0].store(l, Ordering::Release);
        self.target_usage[1].store(r, Ordering::Release);
    }
}

pub(crate) fn reset(path: &Path) -> Result<(), Box<dyn Error>> {
    debug! { println!("Reset: {}", path.display()) }

    let max = fs::read_to_string(path.join("cpuinfo_max_freq"))?;
    fs::write(path.join("scaling_max_freq"), max)?;
    Ok(())
}
