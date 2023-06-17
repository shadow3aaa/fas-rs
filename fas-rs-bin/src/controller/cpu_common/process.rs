use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::thread;

use super::Command;
use super::FrequencyTable;

struct CpuFreq {
    table: FrequencyTable,
    path: PathBuf,
    pos: usize,
}

impl CpuFreq {
    fn new(table: FrequencyTable, write_path: PathBuf) -> Self {
        Self { table, path: write_path, pos: table.len() }
    }
    
    fn prev(&mut self) -> Frequency {
        if pos > 0 {
            pos--;
            self.write();
        }
    }
    
    fn next(&mut self) {
        if pos < self.table.len() {
            pos++;
            self.write();
        }
    }
    
    fn reset(&mut self) {
        self.pos = self.table.len();
        self.write();
    }
    
    fn write(&self) {
        let value = self.table[self.pos].to_string();
        let _ = fs::write(self.path, value);
    }
}

enum Mode {
    Single(CpuFreq),
    Double([CpuFreq; 2])
}

enum Status {
    OnLeft(bool),
    OnRight(bool),
    Single,
}

impl Status {
    fn swap() {
        self = match self {
            OnLeft(b) => Self::OnRight(b),
            OnRight(b) => Self::OnLeft(b),
            Single => Self::Single,
        }
    }
    
    fn swap_target(&mut self, b: bool) {
        self = match self {
            OnLeft(_) => Self::OnRight(b),
            OnRight(_) => Self::OnLeft(b),
            Single => Self::Single,
        }
    }
}

pub(super) fn process_freq(
    tables: Vec<(FrequencyTable, PathBuf)>,
    command_receiver: Receiver<Command>,
) {
    let mut status = Status::Signle;
    let cpufreq = if tables.len() > 1 {
        let table = tables.remove(0);
        let freq_a = CpuFreq::new(table.0, table.1);
        
        let table = tables.remove(0);
        let freq_b = CpuFreq::new(table.0, table.1);
        
        status = Status::Onleft(false);
        
        Mode::Double([freq_a, freq_b])
    } else {
        let table = tables.remove(0);
        let freq = CpuFreq::new(table.0, table.1);
        
        Mode::Single(freq)
    }

    loop {
        if let Ok(command) = command_receiver.recv() {
            match command {
                Command::Pause => {
                    process_pause(&mut cpufreq);
                    thread::park();
                }
                Command::Stop => {
                    process_pause(&mut cpufreq);
                    return;
                }
                Command::Release => {
                    process_release(&table, &mut policy_janked, &mut policy_pos, &mut policy_now)
                }
                Command::Limit => {
                    process_limit(&table, &mut policy_janked, &mut policy_pos, &mut policy_now)
                }
            }
        } else {
            return;
        }
    }
}

fn process_pause(cpufreq: &mut Mode) {
    match cpufreq {
        Single(cpu) => cpu.reset(),
        Double(cpus) => {
            for cpu in cpus {
                cpu.reset();
            }
        }
    }
}

fn process_release(cpufreq: &mut Mode, status: &mut Status) {
    match cpufreq {
        Single(cpu) => cpu.next(),
        Double(cpus) => {
            match status {
                Status::OnLeft(twice) => {
                    if twice {
                        cpus[1].next();
                    } else {
                        cpus[0].next();
                    }
                },
                Status::OnRight(twice) => cpus[1].next();
            }
        }
    }
}

fn process_limit(cpufreq: &mut Mode, status: &mut Status) {
    status.swap();
    match cpufreq {
        Single(cpu) => cpu.next(),
        Double(cpus) => {
            match status {
                Status::OnLeft => cpus[0].prev(),
                Status::OnRight => cpus[1].prev();
            }
        }
    }
}
