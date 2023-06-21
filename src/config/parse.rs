use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use inotify::{Inotify, WatchMask};

use super::GameList;

pub(super) fn wait_and_parse(
    path: PathBuf,
    game_list: Arc<Mutex<GameList>>,
    exit: Arc<AtomicBool>,
) {
    loop {
        if exit.load(Ordering::Acquire) {
            return;
        }

        let mut lock = game_list.lock().unwrap();
        *lock = match parse(&path) {
            Ok(game_list) => game_list,
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    println!("File not found: {}", path.display());
                    thread::sleep(Duration::from_secs(1));
                    continue;
                } else {
                    println!("Failed to read file '{}': {}", path.display(), e);
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            }
        };
        drop(lock);

        // wait until file change
        let mut inotify = Inotify::init().unwrap();
        inotify
            .watches()
            .add(&path, WatchMask::CLOSE_WRITE)
            .unwrap();
        let _ = inotify.read_events_blocking(&mut []);
    }
}

fn parse(path: &Path) -> io::Result<GameList> {
    let raw = fs::read_to_string(path)?;

    let game_list = raw
        .lines()
        .filter_map(|line| {
            let mut split = line.split_whitespace();
            let pkg = split.next()?;
            let fps = split.next()?.parse().ok()?;
            Some((pkg.to_owned(), fps))
        })
        .collect();

    Ok(game_list)
}
