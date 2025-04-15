use std::{
    fs::File,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use zip::{
    ZipWriter,
    result::ZipResult,
    write::{FileOptionExtension, FileOptions},
};

/// Creates a zip archive that contains the files and directories from the specified directory, uses the specified compression level.
pub fn zip_create_from_directory_with_options<F, T>(
    archive_file: &PathBuf,
    directory: &Path,
    cb_file_options: F,
) -> ZipResult<()>
where
    T: FileOptionExtension,
    F: Fn(&PathBuf) -> FileOptions<T>,
{
    let file = File::create(archive_file)?;
    let zip_writer = ZipWriter::new(file);
    create_from_directory_with_options(zip_writer, directory, cb_file_options)
}

fn create_from_directory_with_options<F, T>(
    mut zip_writer: ZipWriter<File>,
    directory: &Path,
    cb_file_options: F,
) -> ZipResult<()>
where
    T: FileOptionExtension,
    F: Fn(&PathBuf) -> FileOptions<T>,
{
    let mut paths_queue: Vec<PathBuf> = vec![];
    paths_queue.push(directory.to_path_buf());

    let mut buffer = Vec::new();

    while let Some(next) = paths_queue.pop() {
        let directory_entry_iterator = std::fs::read_dir(next)?;

        for entry in directory_entry_iterator {
            let entry_path = entry?.path();
            let file_options = cb_file_options(&entry_path);
            let entry_metadata = std::fs::metadata(entry_path.clone())?;
            if entry_metadata.is_file() {
                let mut f = File::open(&entry_path)?;
                f.read_to_end(&mut buffer)?;
                let relative_path = make_relative_path(directory, &entry_path);
                zip_writer.start_file(path_as_string(&relative_path), file_options)?;
                zip_writer.write_all(buffer.as_ref())?;
                buffer.clear();
            } else if entry_metadata.is_dir() {
                let relative_path = make_relative_path(directory, &entry_path);
                zip_writer.add_directory(path_as_string(&relative_path), file_options)?;
                paths_queue.push(entry_path.clone());
            }
        }
    }

    zip_writer.finish()?;
    Ok(())
}

fn make_relative_path(root: &Path, current: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    let root_components = root.components().collect::<Vec<Component>>();
    let current_components = current.components().collect::<Vec<_>>();
    for i in 0..current_components.len() {
        let current_path_component: Component = current_components[i];
        if i < root_components.len() {
            let other: Component = root_components[i];
            if other != current_path_component {
                break;
            }
        } else {
            result.push(current_path_component)
        }
    }
    result
}

fn path_as_string(path: &std::path::Path) -> String {
    let mut path_str = String::new();
    for component in path.components() {
        if let Component::Normal(os_str) = component {
            if !path_str.is_empty() {
                path_str.push('/');
            }
            path_str.push_str(&os_str.to_string_lossy());
        }
    }
    path_str
}
