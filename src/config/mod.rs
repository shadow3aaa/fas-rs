mod read;
mod single;

// 全局配置，可以在任何地方线程安全的访问toml
pub use single::CONFIG;

use std::{
    collections::HashSet,
    fs,
    path::Path,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use likely_stable::LikelyOption;
use log::info;
use parking_lot::RwLock;
use toml::Value;

use read::wait_and_read;

pub type ConfData = RwLock<Value>;
pub struct Config {
    toml: Arc<ConfData>,
    exit: Arc<AtomicBool>,
}

impl Drop for Config {
    fn drop(&mut self) {
        self.exit.store(true, Ordering::Release);
    }
}

impl Config {
    pub fn new(path: &Path) -> Self {
        let ori = fs::read_to_string(path).unwrap();
        let toml = toml::from_str(&ori).unwrap();
        let toml = Arc::new(RwLock::new(toml));
        let exit = Arc::new(AtomicBool::new(false));

        {
            let path = path.to_owned();
            let toml = toml.clone();
            let exit = exit.clone();

            thread::Builder::new()
                .name("ConfigThread".into())
                .spawn(move || wait_and_read(&path, &toml, &exit))
                .unwrap();
        }
        info!("Config watcher started");

        Self { toml, exit }
    }

    pub fn cur_game_fps(&self) -> Option<(String, [u32; 2])> {
        let toml = self.toml.read();
        #[allow(unused)]
        let list = toml
            .get("game_list")
            .and_then_likely(Value::as_table)
            .cloned()
            .unwrap();

        drop(toml); // early-drop

        let pkgs = Self::get_top_pkgname()?;
        let pkg = pkgs.into_iter().find(|key| list.contains_key(key))?;

        let (game, fps_windows) = (&pkg, list.get(&pkg)?.as_array().unwrap());

        let fps_windows: Vec<_> = fps_windows
            .iter()
            .map(|v| u32::try_from(v.as_integer().unwrap()).unwrap())
            .collect();

        Some((game.clone(), fps_windows.as_slice().try_into().unwrap()))
    }

    #[allow(unused)]
    pub fn get_conf(&self, label: &'static str) -> Option<Value> {
        let toml = self.toml.read();
        toml.get("config").unwrap().get(label).cloned()
    }

    fn get_top_pkgname() -> Option<HashSet<String>> {
        let dump = Command::new("dumpsys")
            .args(["window", "visible-apps"])
            .output()
            .ok()?;
        let dump = String::from_utf8_lossy(&dump.stdout).into_owned();

        Some(
            dump.lines()
                .filter(|l| l.contains("package="))
                .map(|p| {
                    p.split_whitespace()
                        .nth(2)
                        .and_then_unlikely(|p| p.split('=').nth(1))
                        .unwrap()
                })
                .zip(
                    dump.lines()
                        .filter(|l| l.contains("canReceiveKeys()"))
                        .map(|k| k.contains("canReceiveKeys()=true")),
                )
                .filter(|(_, k)| *k)
                .map(|(p, _)| p.to_owned())
                .collect(),
        )
    }
}
