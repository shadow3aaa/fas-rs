use std::{fs, path::PathBuf};

use anyhow::Result;

use crate::file_handler::FileHandler;

pub struct CpuTempWatcher {
    nodes: Vec<PathBuf>,
}

impl CpuTempWatcher {
    pub fn new() -> Result<Self> {
        let file_handler = FileHandler::new();
        let mut nodes = Vec::new();
        for device in fs::read_dir("/sys/devices/virtual/thermal")? {
            let device = device?;
            let device_type = device.path().join("type");
            let Ok(device_type) = fs::read_to_string(device_type) else {
                continue;
            };
            if device_type.contains("cpu-")
                || device_type.contains("soc_max")
                || device_type.contains("mtktscpu")
            {
                nodes.push(device.path().join("temp"));
            }
        }

        Ok(Self { nodes })
    }

    pub fn temp(&mut self) -> u64 {
        self.nodes
            .iter()
            .filter_map(|path| fs::read_to_string(path).ok())
            .map(|temp| temp.trim().parse::<u64>().unwrap_or_default())
            .max()
            .unwrap()
    }
}
