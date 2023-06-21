use std::{fs, path::PathBuf, sync::mpsc::Receiver, thread};

use super::Command;
use super::FrequencyTable;

struct CpuFreq {
    table: FrequencyTable,
    path: PathBuf,
    pos: usize,
    jump: [usize; 2],
}

impl CpuFreq {
    fn new(mut table: FrequencyTable, write_path: PathBuf) -> Self {
        table.sort_unstable();
        Self {
            pos: table.len() - 1,
            table,
            path: write_path,
            jump: [1, 3],
        }
    }

    fn prev(&mut self) {
        if self.pos > 1 {
            self.pos -= 1;
            self.write();
        }
        self.jump[0] = 1;
    }

    fn next(&mut self) {
        if self.pos + self.jump[0] < self.table.len() {
            self.pos += self.jump[0];
            self.write();
        }
        
        if self.jump[0] < self.jump[1] {
            self.jump[0] += 1;
        }
    }

    fn reset(&mut self) {
        self.pos = self.table.len() - 1;
        self.write();

        self.jump[0] = 1;
    }

    fn write(&self) {
        use std::{fs::set_permissions, os::unix::fs::PermissionsExt};
        let value = self.table[self.pos].to_string();
        set_permissions(&self.path, PermissionsExt::from_mode(0o644)).unwrap();
        fs::write(&self.path, value).unwrap();
        set_permissions(&self.path, PermissionsExt::from_mode(0o444)).unwrap();
    }
}

enum Mode {
    Single(CpuFreq),
    Double([CpuFreq; 2]),
}

enum Status {
    OnLeft,
    OnRight,
}

impl Status {
    fn swap(self) -> Self {
        match self {
            Self::OnLeft => Self::OnRight,
            Self::OnRight => Self::OnLeft,
        }
    }
}

pub(super) fn process_freq(
    mut tables: Vec<(FrequencyTable, PathBuf)>,
    command_receiver: Receiver<Command>,
) {
    let mut status = None;
    let mut cpufreq = if tables.len() > 1 {
        let table = tables.remove(0);
        let freq_a = CpuFreq::new(table.0, table.1);

        let table = tables.remove(0);
        let freq_b = CpuFreq::new(table.0, table.1);

        status = Some(Status::OnLeft);

        Mode::Double([freq_a, freq_b])
    } else {
        let table = tables.remove(0);
        let freq = CpuFreq::new(table.0, table.1);

        Mode::Single(freq)
    };

    loop {
        let command = command_receiver.recv().unwrap();
        match command {
            Command::Pause => {
                process_pause(&mut cpufreq);
                // count清空管道
                let _ = command_receiver.try_iter().count();
                thread::park();
            }
            Command::Stop => {
                process_pause(&mut cpufreq);
                return;
            }
            Command::Release => status = process_release(&mut cpufreq, status),
            Command::Limit => status = process_limit(&mut cpufreq, status),
        }
    }
}

fn process_pause(cpufreq: &mut Mode) {
    match cpufreq {
        Mode::Single(cpu) => cpu.reset(),
        Mode::Double(cpus) => {
            for cpu in cpus {
                cpu.reset();
            }
        }
    }
}

fn process_release(cpufreq: &mut Mode, status: Option<Status>) -> Option<Status> {
    match cpufreq {
        Mode::Single(cpu) => {
            cpu.next();
            None
        }
        Mode::Double(cpus) => {
            let status = status.unwrap();
            match status {
                Status::OnLeft => cpus[0].next(),
                Status::OnRight => cpus[1].next(),
            }
            Some(status.swap())
        }
    }
}

fn process_limit(cpufreq: &mut Mode, status: Option<Status>) -> Option<Status> {
    match cpufreq {
        Mode::Single(cpu) => {
            cpu.next();
            None
        }
        Mode::Double(cpus) => {
            let status = status.unwrap().swap();
            match status {
                Status::OnLeft => cpus[0].prev(),
                Status::OnRight => cpus[1].prev(),
            }
            Some(status)
        }
    }
}
