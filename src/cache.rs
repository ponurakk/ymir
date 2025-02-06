use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::PathBuf,
};

use anyhow::{bail, Context};

use crate::{
    config::Cache,
    projects::{Project, ProjectLanguage},
    utils::GitInfo,
};

const MAGIC: &[u8; 4] = b"YMIR";
const VERSION: u8 = 1;

pub trait CacheSerializer {
    fn serialize(&self) -> anyhow::Result<Vec<u8>>;
    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl CacheSerializer for Cache {
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(MAGIC);
        buffer.push(VERSION);

        // Projects len
        buffer.extend_from_slice(&u16::try_from(self.projects.len())?.to_le_bytes());

        for project in &self.projects {
            buffer.extend_from_slice(&project.serialize()?);
        }

        Ok(buffer)
    }

    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let mut magic = [0u8; 4];
        cursor
            .read_exact(&mut magic)
            .with_context(|| "Failed to read magic")?;
        if &magic != MAGIC {
            bail!("Invalid magic value");
        }

        let mut version = [0u8; 1];
        cursor
            .read_exact(&mut version)
            .with_context(|| "Failed to read version")?;
        if version[0] != VERSION {
            bail!("Invalid version. Found: {}, eurrent {VERSION}", version[0]);
        }

        let mut len_bytes = [0u8; 2];
        cursor
            .read_exact(&mut len_bytes)
            .with_context(|| "Failed to read projects length")?;
        let projects_len = u16::from_le_bytes(len_bytes) as usize;

        let mut projects: Vec<Project> = Vec::new();

        for _ in 0..projects_len {
            projects.push(Project::deserialize(cursor)?);
        }

        Ok(Self { projects })
    }
}

impl CacheSerializer for Project {
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();

        // Path
        let path = self.path.to_string_lossy();
        buffer.extend_from_slice(&u16::try_from(path.len())?.to_le_bytes());
        buffer.extend_from_slice(&path.to_string().as_bytes());

        // Size
        buffer.extend_from_slice(&self.size.to_le_bytes());

        // Git Info
        buffer.extend_from_slice(&GitInfo::serialize(&self.git_info)?);
        buffer.extend_from_slice(&self.languages.serialize()?);
        buffer.extend_from_slice(&ProjectLanguage::serialize(&self.languages_total)?);

        Ok(buffer)
    }

    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let mut len_bytes = [0u8; 2];

        // Path
        cursor
            .read_exact(&mut len_bytes)
            .with_context(|| "Failed to read path len")?;
        let path_len = u16::from_le_bytes(len_bytes) as usize;
        let mut path = vec![0u8; path_len];
        cursor
            .read_exact(&mut path)
            .with_context(|| "Failed to read path")?;
        let path = PathBuf::from(String::from_utf8(path)?);

        // Size
        let mut size = u64::MAX.to_le_bytes();
        cursor
            .read_exact(&mut size)
            .with_context(|| "Failed to read size")?;
        let size = u64::from_le_bytes(size);

        let git_info = GitInfo::deserialize(cursor)?;
        let languages = HashMap::deserialize(cursor)?;
        let languages_total = ProjectLanguage::deserialize(cursor)?;

        Ok(Self {
            path,
            size,
            git_info,
            languages,
            languages_total,
        })
    }
}

impl CacheSerializer for GitInfo {
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();

        // Remote url
        buffer.extend_from_slice(&u16::try_from(self.remote_url.len())?.to_le_bytes());
        buffer.extend_from_slice(&self.remote_url.to_string().as_bytes());

        // Init date
        buffer.extend_from_slice(&u16::try_from(self.init_date.len())?.to_le_bytes());
        buffer.extend_from_slice(&self.init_date.to_string().as_bytes());

        // Last commit date
        buffer.extend_from_slice(&u16::try_from(self.last_commit_date.len())?.to_le_bytes());
        buffer.extend_from_slice(&self.last_commit_date.to_string().as_bytes());

        // Last commit msg
        buffer.extend_from_slice(&u16::try_from(self.last_commit_msg.len())?.to_le_bytes());
        buffer.extend_from_slice(&self.last_commit_msg.to_string().as_bytes());

        // Commit count
        buffer.extend_from_slice(&self.commit_count.to_le_bytes());

        Ok(buffer)
    }

    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let mut len_bytes = [0u8; 2];
        cursor.read_exact(&mut len_bytes)?;

        // Remote url
        let remote_url_len = u16::from_le_bytes(len_bytes) as usize;
        let mut remote_url = vec![0u8; remote_url_len];
        cursor
            .read_exact(&mut remote_url)
            .with_context(|| "Failed to read remote url")?;
        let remote_url = String::from_utf8(remote_url).with_context(|| "Invalid UTF-8 key")?;

        // Init date
        cursor.read_exact(&mut len_bytes)?;
        let init_date_len = u16::from_le_bytes(len_bytes) as usize;
        let mut init_date_url = vec![0u8; init_date_len];
        cursor
            .read_exact(&mut init_date_url)
            .with_context(|| "Failed to read init date")?;
        let init_date = String::from_utf8(init_date_url).with_context(|| "Invalid UTF-8 key")?;

        // Last commit date
        cursor.read_exact(&mut len_bytes)?;
        let last_commit_date_len = u16::from_le_bytes(len_bytes) as usize;
        let mut last_commit_date = vec![0u8; last_commit_date_len];
        cursor
            .read_exact(&mut last_commit_date)
            .with_context(|| "Failed to read last commit date")?;
        let last_commit_date =
            String::from_utf8(last_commit_date).with_context(|| "Invalid UTF-8 key")?;

        // Last commit msg
        cursor.read_exact(&mut len_bytes)?;
        let last_commit_msg_len = u16::from_le_bytes(len_bytes) as usize;
        let mut last_commit_msg = vec![0u8; last_commit_msg_len];
        cursor
            .read_exact(&mut last_commit_msg)
            .with_context(|| "Failed to read last commit msg")?;
        let last_commit_msg =
            String::from_utf8(last_commit_msg).with_context(|| "Invalid UTF-8 key")?;

        let mut commit_count = u32::MAX.to_le_bytes();
        cursor.read_exact(&mut commit_count)?;
        let commit_count = u32::from_le_bytes(commit_count);

        Ok(Self {
            remote_url,
            init_date,
            last_commit_date,
            last_commit_msg,
            commit_count,
        })
    }
}

impl CacheSerializer for ProjectLanguage {
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.extend_from_slice(&self.files.to_le_bytes());
        buffer.extend_from_slice(&self.lines.to_le_bytes());
        buffer.extend_from_slice(&self.code.to_le_bytes());
        buffer.extend_from_slice(&self.comments.to_le_bytes());
        buffer.extend_from_slice(&self.blanks.to_le_bytes());

        Ok(buffer)
    }

    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let mut files = u32::MAX.to_le_bytes();
        cursor.read_exact(&mut files)?;
        let files = u32::from_le_bytes(files);

        let mut lines = u32::MAX.to_le_bytes();
        cursor.read_exact(&mut lines)?;
        let lines = u32::from_le_bytes(lines);

        let mut code = u32::MAX.to_le_bytes();
        cursor.read_exact(&mut code)?;
        let code = u32::from_le_bytes(code);

        let mut comments = u32::MAX.to_le_bytes();
        cursor.read_exact(&mut comments)?;
        let comments = u32::from_le_bytes(comments);

        let mut blanks = u32::MAX.to_le_bytes();
        cursor.read_exact(&mut blanks)?;
        let blanks = u32::from_le_bytes(blanks);

        Ok(Self {
            files,
            lines,
            code,
            comments,
            blanks,
        })
    }
}

impl<T> CacheSerializer for HashMap<String, T>
where
    T: CacheSerializer,
{
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.extend_from_slice(&u16::try_from(self.len())?.to_le_bytes());
        for (key, value) in self {
            // Key
            buffer.extend_from_slice(&u16::try_from(key.len())?.to_le_bytes());
            buffer.extend_from_slice(key.as_bytes());

            // Value
            buffer.extend_from_slice(&T::serialize(value)?);
        }

        Ok(buffer)
    }

    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let mut len_bytes = [0u8; 2];

        // Hashmap count
        let mut hashmap_len = u16::MAX.to_le_bytes();
        cursor
            .read_exact(&mut hashmap_len)
            .with_context(|| "Failed to read hashmap len")?;
        let hashmap_len = u16::from_le_bytes(hashmap_len);

        let mut hashmap = HashMap::new();

        for _ in 0..hashmap_len {
            // KEY
            cursor
                .read_exact(&mut len_bytes)
                .with_context(|| "Failed to read key len")?;
            let key_len = u16::from_le_bytes(len_bytes) as usize;
            let mut key = vec![0u8; key_len];
            cursor
                .read_exact(&mut key)
                .with_context(|| "Failed to read key")?;

            let key = String::from_utf8(key)?;
            let value = T::deserialize(cursor)?;

            hashmap.insert(key, value);
        }

        Ok(hashmap)
    }
}
