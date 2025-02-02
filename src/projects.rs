//! Functions for finding projects

use std::{ffi::OsStr, fmt::Display, path::PathBuf};

use tokei::{Config, Languages};
use walkdir::{DirEntry, WalkDir};

use crate::utils::{format_bytes, get_git_info, get_size, GitInfo};

#[derive(Debug)]
pub struct Project {
    pub path: PathBuf,
    pub size: u64,
    pub git_info: GitInfo,
    pub languages: Languages,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Project Name: {}\nPath: {}\nSize: {}\nCreated At: {}\nModified At: {}\n\n# Git:\nLast Commit: {}\nCommits: {}\nRemote: {}",
            self.path
                .file_name()
                .map_or("Failed to get file name", |v| v
                    .to_str()
                    .unwrap_or_default()),
            self.path.display(),
            format_bytes(self.size),
            self.git_info.init_date,
            self.git_info.last_commit_date,
            self.git_info.last_commit_msg,
            self.git_info.commit_count,
            self.git_info.remote_url,
        )
    }
}

impl Project {
    pub fn new(path: PathBuf, size: u64, languages: Languages) -> Self {
        let git_info = get_git_info(&path).unwrap_or_default();

        Self {
            path,
            size,
            git_info,
            languages,
        }
    }
}

/// Checks if the entry is a build directory
fn is_build(entry: &DirEntry, ignore_dirs: &[String]) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|s| ignore_dirs.contains(&s.to_string()))
}

/// Returns a list of directories that contain a `.git` directory
pub fn find(path: PathBuf, ignore_dirs: &[String]) -> Vec<Project> {
    let mut paths: Vec<Project> = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_build(e, ignore_dirs))
        .filter_map(Result::ok)
    {
        if entry.path().file_name() != Some(OsStr::new(".git")) {
            continue;
        }

        let Some(parent) = entry.path().parent() else {
            // TODO: Add error log
            eprintln!("Failed to get parent of directory");
            continue;
        };

        let mut languages = Languages::new();
        languages.get_statistics(&[parent], &[], &Config::default());

        let size = get_size(parent).unwrap_or(0);
        paths.push(Project::new(parent.to_path_buf(), size, languages));
        eprintln!("{} - {}", paths.len(), parent.display());
    }

    paths
}
