use std::{
    error::Error,
    fs::{self, set_permissions},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
};

pub struct WritePool {
    workers: Vec<(Sender<Command>, Arc<AtomicUsize>)>,
}

enum Command {
    Write(PathBuf, String),
    Exit,
}

impl Drop for WritePool {
    fn drop(&mut self) {
        self.workers.iter().for_each(|(sender, _)| {
            let _ = sender.send(Command::Exit);
        });
    }
}

impl WritePool {
    pub fn new(worker_count: usize) -> Self {
        let mut workers = Vec::with_capacity(worker_count);
        for _ in 0..worker_count {
            let (sender, receiver) = mpsc::channel();
            let heavy = Arc::new(AtomicUsize::new(0));
            let heavy_clone = heavy.clone();
            thread::spawn(move || write_thread(receiver, heavy_clone));
            workers.push((sender, heavy));
        }

        Self { workers }
    }

    pub fn write(&mut self, path: &Path, value: &str) -> Result<(), Box<dyn Error>> {
        let (best_worker, heavy) = self
            .workers
            .iter()
            .min_by_key(|(_, heavy)| heavy.load(Ordering::Acquire))
            .unwrap();

        let new_heavy = heavy.load(Ordering::Acquire) + 1;
        heavy.store(new_heavy, Ordering::Release); // 完成一个任务负载计数加一

        best_worker.send(Command::Write(path.to_owned(), value.to_owned()))?;
        Ok(())
    }
}

fn write_thread(receiver: Receiver<Command>, heavy: Arc<AtomicUsize>) {
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
        let new_heavy = heavy.load(Ordering::Acquire).saturating_sub(1);
        heavy.store(new_heavy, Ordering::Release); // 完成一个任务负载计数器减一
    }
}
