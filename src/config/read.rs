use std::{
    fs,
    path::Path,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use inotify::{Inotify, WatchMask};

use super::ConfData;
use crate::{debug, ThisResult};

pub(super) fn wait_and_read(path: &Path, toml: &Arc<ConfData>, exit: &Arc<AtomicBool>) {
    let mut retry_count = 0;
    loop {
        if exit.load(Ordering::Acquire) {
            return;
        }

        if retry_count > 10 {
            debug! {
                eprintln!("Too many read config retries");
            }
            process::exit(1);
        }

        let ori = match fs::read_to_string(path) {
            Ok(s) => {
                retry_count = 0;
                s
            }
            #[allow(unused_variables)]
            Err(e) => {
                debug! {
                    println!("Failed to read file '{}': {}", path.display(), e);
                }
                retry_count += 1;
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };
        *toml.write() = toml::from_str(&ori).this_unwrap();
        debug! {
            println!("{:#?}", *toml.read());
        }

        // wait until file change
        let mut inotify = Inotify::init().this_unwrap();
        inotify
            .watches()
            .add(path, WatchMask::CLOSE_WRITE)
            .this_unwrap();
        let _ = inotify.read_events_blocking(&mut []);
    }
}