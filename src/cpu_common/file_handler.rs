use std::{
    collections::{hash_map::Entry, HashMap},
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};

use anyhow::Result;

#[derive(Debug)]
pub struct FileHandler {
    files: HashMap<PathBuf, File>,
}

impl FileHandler {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn read_to_string(&mut self, path: impl AsRef<Path>) -> Result<String> {
        let mut string = String::new();
        match self.files.entry(path.as_ref().to_path_buf()) {
            Entry::Occupied(entry) => {
                let mut string = String::new();
                entry.get().read_to_string(&mut string)?;
            }
            Entry::Vacant(entry) => {
                let mut file = File::open(path.as_ref())?;
                file.read_to_string(&mut string)?;
                entry.insert(file);
            }
        }

        Ok(string)
    }

    pub fn write(&mut self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Result<()> {
        match self.files.entry(path.as_ref().to_path_buf()) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().write_all(content.as_ref())?;
            }
            Entry::Vacant(entry) => {
                let mut file = File::create(path)?;
                file.write_all(content.as_ref())?;
                entry.insert(file);
            }
        }

        Ok(())
    }
}
