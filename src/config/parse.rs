use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use inotify::{Inotify, WatchMask};

use super::GameList;

pub(super) fn wait_and_parse(
    path: PathBuf,
    game_list: Arc<Mutex<GameList>>,
    exit: Arc<AtomicBool>,
) {
    let mut inotify = Inotify::init().unwrap();
    inotify.watches().add(&path, WatchMask::MODIFY).unwrap();

    loop {
        if exit.load(Ordering::Acquire) {
            return;
        }

        let mut lock = game_list.lock().unwrap();
        *lock = parse(&path);
        drop(lock);

        // wait until file change
        let _ = inotify.read_events_blocking(&mut []);
    }
}

fn parse(path: &Path) -> GameList {
    let raw = fs::read_to_string(path).unwrap();

    raw.lines()
        .filter_map(|line| {
            let mut split = line.split_whitespace();
            let pkg = split.next()?;
            let fps = split.next()?.parse().ok()?;
            Some((pkg.to_owned(), fps))
        })
        .collect()
}
