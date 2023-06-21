mod process;
pub(crate) mod write_pool;

use fas_rs_fw::prelude::*;
use std::{
    fs,
    path::Path,
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

use process::*;

const CPUFREQ: &str = "/sys/devices/system/cpu/cpufreq";

pub(crate) type Frequency = usize;
pub(crate) type FrequencyTable = Vec<Frequency>;
pub struct CpuCommon {
    command_sender: Sender<Command>,
    thread_handle: JoinHandle<()>,
}

pub(crate) enum Command {
    Stop,
    Release,
    Limit,
    Pause,
}

impl Drop for CpuCommon {
    fn drop(&mut self) {
        let _ = self.command_sender.send(Command::Stop);
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
                .split_whitespace()
                .filter_map(|freq| freq.parse().ok())
                .collect();

            table.push((this_table, path.join("scaling_max_freq")));
        }

        // 按policy降序排列
        table.sort_by(|a, b| {
            let name_a = a.1.parent().unwrap().file_name().unwrap().to_str().unwrap();
            let name_b = b.1.parent().unwrap().file_name().unwrap().to_str().unwrap();

            let num_a: usize = name_a.split("policy").nth(1).unwrap().parse().unwrap();
            let num_b: usize = name_b.split("policy").nth(1).unwrap().parse().unwrap();
            num_b.cmp(&num_a)
        });
        table.truncate(2); // 保留后两个集群即可

        let (command_sender, command_receiver) = mpsc::channel();

        let thread_handle = thread::spawn(move || process_freq(table, command_receiver));

        Ok(Self {
            command_sender,
            thread_handle,
        })
    }

    fn limit(&self) {
        self.command_sender.send(Command::Limit).unwrap();
    }

    fn release(&self) {
        self.command_sender.send(Command::Release).unwrap();
    }

    fn plug_in(&self) -> Result<(), Box<dyn Error>> {
        self.thread_handle.thread().unpark();
        Ok(())
    }

    fn plug_out(&self) -> Result<(), Box<dyn Error>> {
        self.command_sender.send(Command::Pause).unwrap();
        Ok(())
    }
}
