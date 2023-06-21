use std::{
    error::Error,
    fs::{self, set_permissions},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

pub struct WritePool {
    pos: usize,
    senders: Vec<Sender<Command>>,
}

enum Command {
    Write(PathBuf, String),
    Exit,
}

impl Drop for WritePool {
    fn drop(&mut self) {
        self.senders.iter().for_each(|sender| {
            let _ = sender.send(Command::Exit);
        });
    }
}

impl WritePool {
    pub fn new(thread_count: usize) -> Self {
        let mut senders = Vec::with_capacity(thread_count);
        for _ in 0..thread_count {
            let (sender, receiver) = mpsc::channel();
            thread::spawn(move || write_thread(receiver));
            senders.push(sender);
        }

        Self { pos: 0, senders }
    }

    pub fn write(&mut self, path: &Path, value: &str) -> Result<(), Box<dyn Error>> {
        self.senders[self.pos].send(Command::Write(path.to_owned(), value.to_owned()))?;

        if self.pos < self.senders.len() - 1 {
            self.pos += 1;
        } else {
            self.pos = 0;
        }

        Ok(())
    }
}

fn write_thread(receiver: Receiver<Command>) {
    loop {
        if let Ok(command) = receiver.recv() {
            match command {
                Command::Write(path, value) => {
                    set_permissions(&path, PermissionsExt::from_mode(0o644)).unwrap();
                    fs::write(&path, value).unwrap();
                    set_permissions(&path, PermissionsExt::from_mode(0o444)).unwrap()
                }
                Command::Exit => return,
            }
        } else {
            return;
        }
    }
}
