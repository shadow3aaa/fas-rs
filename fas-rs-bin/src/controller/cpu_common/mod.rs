mod process;

use std::sync::mpsc::{self, SyncSender};
use std::fs;
use std::thread::{self, JoinHandle};
use std::path::Path;

use fas_rs_fw::prelude::*;

use process::*;

const CPUFREQ: &str = "/sys/devices/system/cpu/cpufreq";

pub(crate) type Frequency = usize;
pub(crate) type FrequencyTable = Vec<Frequency>;
pub struct CpuCommon {
    command_sender: SyncSender<Command>,
    thread_handle: JoinHandle<()>,
}

pub(crate) enum Command {
    Pause, // 暂停
    Stop, // 结束
    Release,
    Limit,
}

impl Drop for CpuCommon {
    fn drop(&mut self) {
        self.command_sender.send(Command::Stop).unwrap();
    }
}

impl VirtualPerformanceController for CpuCommon {
    fn support() -> bool
    where
        Self: Sized,
    {
        Path::new(CPUFREQ).exists()
    }

    fn new() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut table = Vec::with_capacity(2);

        let cpufreq = fs::read_dir(CPUFREQ)?;
        for policy in cpufreq {
            let path = policy?.path();
            let table_path = path.join("scaling_available_frequencies");
            let this_table: FrequencyTable = fs::read_to_string(table_path)?
                .trim()
                .split_whitespace()
                .filter_map(|freq| freq.parse().ok())
                .collect();

            table.push((this_table, path.join("scaling_max_freq")));
        }

        table.reverse();
        table.truncate(2); // 保留后两个集群即可

        let (command_sender, command_receiver) = mpsc::sync_channel(1);
        let thread_handle = thread::spawn(|| process_freq(table, command_receiver));

        Ok(Self { command_sender, thread_handle })
    }

    fn limit(&self) {
        let _ = self.command_sender.try_send(Command::Limit);
    }

    fn release(&self) {
        let _ = self.command_sender.try_send(Command::Release);
    }

    fn plug_in(&self) -> Result<(), Box<dyn Error>> {
        self.thread_handle.thread().unpark();
        Ok(())
    }

    fn plug_out(&self) -> Result<(), Box<dyn Error>> {
        self.command_sender.try_send(Command::Pause)?;
        Ok(())
    }
}