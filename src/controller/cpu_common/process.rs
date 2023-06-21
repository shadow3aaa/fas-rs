use std::{path::PathBuf, sync::mpsc::Receiver, thread};

use super::write_pool::WritePool;
use super::Command;
use super::FrequencyTable;

struct CpuFreq {
    table: FrequencyTable,
    path: PathBuf,
    pool: WritePool,
    pos: usize,
    jump: [usize; 2],
}

impl CpuFreq {
    fn new(mut table: FrequencyTable, write_path: PathBuf) -> Self {
        table.sort_unstable();
        let pool = WritePool::new(2);
        Self {
            pos: table.len() - 1,
            table,
            pool,
            path: write_path,
            jump: [1, 4],
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

    fn write(&mut self) {
        let _ = self
            .pool
            .write(&self.path, &self.table[self.pos].to_string());
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
    fn swap(&mut self) {
        *self = match *self {
            Self::OnLeft => Self::OnRight,
            Self::OnRight => Self::OnLeft,
        };
    }
}

pub(super) fn process_freq(
    mut tables: Vec<(FrequencyTable, PathBuf)>,
    command_receiver: Receiver<Command>,
) {
    let mut status = Status::OnLeft;
    let mut cpufreq = if tables.len() > 1 {
        let table = tables.remove(0);
        let freq_a = CpuFreq::new(table.0, table.1);

        let table = tables.remove(0);
        let freq_b = CpuFreq::new(table.0, table.1);

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
                thread::park();
                // count清空管道
                let _ = command_receiver.try_iter().count();
            }
            Command::Stop => {
                process_pause(&mut cpufreq);
                return;
            }
            Command::Release => process_release(&mut cpufreq, &mut status),
            Command::Limit => process_limit(&mut cpufreq, &mut status),
        }
    }
}

fn process_pause(cpufreq: &mut Mode) {
    match cpufreq {
        Mode::Single(cpu) => cpu.reset(),
        Mode::Double(cpus) => cpus.iter_mut().for_each(|cpu| cpu.reset()),
    }
}

fn process_release(cpufreq: &mut Mode, status: &mut Status) {
    match cpufreq {
        Mode::Single(cpu) => cpu.next(),
        Mode::Double(cpus) => {
            match status {
                Status::OnLeft => cpus[0].next(),
                Status::OnRight => cpus[1].next(),
            }
            status.swap();
        }
    }
}

fn process_limit(cpufreq: &mut Mode, status: &mut Status) {
    match cpufreq {
        Mode::Single(cpu) => cpu.next(),
        Mode::Double(cpus) => {
            status.swap();
            match status {
                Status::OnLeft => cpus[0].prev(),
                Status::OnRight => cpus[1].prev(),
            }
        }
    }
}
