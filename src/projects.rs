use std::{ffi::OsStr, path::PathBuf};

use walkdir::{DirEntry, WalkDir};

fn is_build(entry: &DirEntry, ignore_dirs: &[String]) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|s| ignore_dirs.contains(&s.to_string()))
}

pub fn find(path: PathBuf, ignore_dirs: &[String]) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_build(e, ignore_dirs))
        .filter_map(Result::ok)
    {
        if entry.path().file_name() != Some(OsStr::new(".git")) {
            continue;
        }

        let Some(parent) = entry.path().parent() else {
            // TODO: Add error log here
            eprintln!("Failed to get parent of directory");
            continue;
        };

        paths.push(parent.to_path_buf());
    }

    paths
}
