//! Functions for finding projects

use std::{collections::HashMap, ffi::OsStr, fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};
use tokei::{Config, Languages};
use walkdir::{DirEntry, WalkDir};

use crate::utils::{format_bytes, get_git_info, get_size, GitInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub path: PathBuf,
    pub size: u64,
    pub git_info: GitInfo,
    pub languages: HashMap<String, ProjectLanguage>,
    pub languages_total: ProjectLanguage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLanguage {
    pub files: usize,
    pub lines: usize,
    pub code: usize,
    pub comments: usize,
    pub blanks: usize,
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
    pub fn new(
        path: PathBuf,
        size: u64,
        languages: HashMap<String, ProjectLanguage>,
        languages_total: ProjectLanguage,
    ) -> Self {
        let git_info = get_git_info(&path).unwrap_or_default();

        Self {
            path,
            size,
            git_info,
            languages,
            languages_total,
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

        let total = languages.total();
        let total: ProjectLanguage = ProjectLanguage {
            files: total.reports.len(),
            lines: total.lines(),
            code: total.code,
            comments: total.comments,
            blanks: total.blanks,
        };

        let languages: HashMap<String, ProjectLanguage> = languages
            .into_iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    ProjectLanguage {
                        files: value.reports.len(),
                        lines: value.lines(),
                        code: value.code,
                        comments: value.comments,
                        blanks: value.blanks,
                    },
                )
            })
            .collect();

        let size = get_size(parent).unwrap_or(0);
        paths.push(Project::new(parent.to_path_buf(), size, languages, total));
        eprintln!("{} - {}", paths.len(), parent.display());
    }

    paths
}

pub fn find_from_cache(projects: Vec<PathBuf>) -> Vec<Project> {
    let mut paths: Vec<Project> = Vec::new();

    for path in projects {
        let mut languages = Languages::new();
        languages.get_statistics(&[&path], &[], &Config::default());

        let total = languages.total();
        let total: ProjectLanguage = ProjectLanguage {
            files: total.reports.len(),
            lines: total.lines(),
            code: total.code,
            comments: total.comments,
            blanks: total.blanks,
        };

        let languages: HashMap<String, ProjectLanguage> = languages
            .into_iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    ProjectLanguage {
                        files: value.reports.len(),
                        lines: value.lines(),
                        code: value.code,
                        comments: value.comments,
                        blanks: value.blanks,
                    },
                )
            })
            .collect();

        let size = get_size(&path).unwrap_or(0);
        paths.push(Project::new(path.to_path_buf(), size, languages, total));
        eprintln!("{} - {}", paths.len(), path.display());
    }

    paths
}
