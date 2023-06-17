use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::thread;

use super::FrequencyTable;
use super::Command;

pub(super) fn process_freq(table: Vec<(FrequencyTable, PathBuf)>, command_receiver: Receiver<Command>) {
    let table = table;
    let mut policy_flag = None;

    loop {
        if let Ok(command) = command_receiver.recv() {
            match command {
                Command::Pause => {
                    process_pause(&table, &mut policy_flag);
                    thread::park();
                },
                Command::Stop => return,
                Command::Release => process_release(&table, &mut policy_flag),
                Command::Limit => process_limit(&table, &mut policy_flag),
            }
        } else {
            return;
        }
    }
}

fn process_pause(table: &Vec<(FrequencyTable, PathBuf)>, flag: &mut Option<bool>) {
    *flag = None;
    for (freqtable, path) in table {
        let max_freq = freqtable.last().unwrap();
        let _ = fs::write(path, max_freq.to_string());
    }
}

fn process_release(table: &Vec<(FrequencyTable, PathBuf)>, flag: &mut Option<bool>) {
    todo!()
}

fn process_limit(table: &Vec<(FrequencyTable, PathBuf)>, flag: &mut Option<bool>) {
    todo!()
}