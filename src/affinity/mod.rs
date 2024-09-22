mod applyer_thread;
mod helper_thread;

use std::{
    sync::mpsc::{self, Sender},
    thread::{self},
};

use helper_thread::{affinity_helper, Command};

pub struct Affinity {
    sx: Sender<Command>,
}

impl Affinity {
    pub fn new() -> Self {
        let (sx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("AffinityHelper".into())
            .spawn(move || affinity_helper(&rx))
            .unwrap();
        Self { sx }
    }

    pub fn attach(&self, pid: i32) {
        let _ = self.sx.send(Command::Attach(pid));
    }

    pub fn detach(&self) {
        let _ = self.sx.send(Command::Detach);
    }

    pub fn apply(&self) {
        let _ = self.sx.send(Command::Apply);
    }
}
