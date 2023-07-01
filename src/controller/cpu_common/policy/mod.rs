mod cycles;
mod schedule;

use std::{
    error::Error,
    fs::{self, set_permissions},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use cpu_cycles_reader::Cycles;
use parking_lot::RwLock;

use crate::debug;
use cycles::*;
use schedule::Schedule;

pub struct Policy {
    path: PathBuf,
    target_diff: Arc<RwLock<Cycles>>,
    pause: Arc<AtomicBool>,
    exit: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

impl Drop for Policy {
    fn drop(&mut self) {
        self.resume();
        self.exit.store(true, Ordering::Release);
        let _ = reset(&self.path);
    }
}

impl Policy {
    pub fn new(policy_path: &Path, burst_max: usize) -> Self {
        let (schedule, target_diff) = Schedule::new(policy_path, burst_max);

        let pause = Arc::new(AtomicBool::new(false));
        let exit = Arc::new(AtomicBool::new(false));

        let pause_clone = pause.clone();
        let exit_clone = exit.clone();
        let path_clone = policy_path.to_owned();
        let handle =
            thread::spawn(move || cycles_thread(&path_clone, schedule, pause_clone, exit_clone));

        Self {
            path: policy_path.to_owned(),
            target_diff,
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
    }

    pub fn set_target_diff(&self, c: Cycles) {
        *self.target_diff.write() = c;
    }
}

pub(crate) fn reset(path: &Path) -> Result<(), Box<dyn Error>> {
    debug! { println!("Reset: {}", path.display()) }

    let max = fs::read_to_string(path.join("cpuinfo_max_freq"))?;
    let path = path.join("scaling_max_freq");
    set_permissions(&path, PermissionsExt::from_mode(0o644))?;
    fs::write(path, max)?;
    Ok(())
}
