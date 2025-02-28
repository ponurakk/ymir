use std::{fs::read_dir, path::Path};

use chrono::{DateTime, Local};
use git2::Repository;

pub fn format_bytes(bytes: u64) -> String {
    let sizes = ["B", "K", "M", "G", "T", "P", "E"];
    #[allow(clippy::cast_precision_loss)]
    let mut size = bytes as f64;
    let mut index = 0;

    while size >= 1024.0 && index < sizes.len() - 1 {
        size /= 1024.0;
        index += 1;
    }

    format!("{:.1}{}", size, sizes[index])
}

pub fn get_size<P>(path: P) -> anyhow::Result<u64>
where
    P: AsRef<Path>,
{
    let path_metadata = path.as_ref().symlink_metadata()?;

    let mut size_in_bytes = 0;

    if path_metadata.is_dir() {
        for entry in read_dir(&path)? {
            let entry = entry?;
            let entry_metadata = entry.metadata()?;

            if entry_metadata.is_dir() {
                size_in_bytes += get_size(entry.path())?;
            } else {
                size_in_bytes += entry_metadata.len();
            }
        }
    } else {
        size_in_bytes = path_metadata.len();
    }

    Ok(size_in_bytes)
}

#[derive(Debug, Clone, Default)]
pub struct GitInfo {
    pub remote_url: Option<String>,
    pub init_date: u32,
    pub last_commit_date: u32,
    pub last_commit_msg: Option<String>,
    pub commit_count: u32,
}

pub fn get_git_info(repo_path: &Path) -> anyhow::Result<GitInfo> {
    let repo = Repository::open(repo_path)?;

    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(String::from));

    let mut revwalk = repo.revwalk()?;
    if revwalk.push_head().is_err() {
        // TODO: Log error
        return Ok(GitInfo::default());
    }

    revwalk.set_sorting(git2::Sort::REVERSE)?;
    let first_commit_id = revwalk.next().and_then(Result::ok);
    let last_commit_id = revwalk.last().and_then(Result::ok).or(first_commit_id);

    let mut first_commit_time: Option<i64> = None;

    if let Some(first_id) = first_commit_id {
        let first_commit = repo.find_commit(first_id)?;
        first_commit_time = Some(first_commit.time().seconds());
    }

    let mut last_commit_time: Option<i64> = None;
    let mut last_commit_message: Option<String> = None;
    if let Some(last_id) = last_commit_id {
        let last_commit = repo.find_commit(last_id)?;
        last_commit_time = Some(last_commit.time().seconds());
        last_commit_message = Some(
            last_commit
                .message()
                .map_or("No message", |v| v.lines().next().unwrap_or("No message"))
                .to_string(),
        );
    }

    let mut revwalk_count = repo.revwalk()?;
    revwalk_count.push_head()?; // Push HEAD so walker sees commits
    let commit_count = u32::try_from(revwalk_count.count())?;

    Ok(GitInfo {
        remote_url,
        init_date: format_time(first_commit_time),
        last_commit_date: format_time(last_commit_time),
        last_commit_msg: last_commit_message.as_ref().map(|v| v.trim().to_string()),
        commit_count,
    })
}

fn format_time(timestamp: Option<i64>) -> u32 {
    timestamp
        .and_then(|t| DateTime::from_timestamp(t, 0))
        .map_or(0, |dt| {
            u32::try_from(dt.with_timezone(&Local).timestamp()).unwrap_or_default()
        })
}
