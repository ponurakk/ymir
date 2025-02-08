use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::PathBuf,
};

use anyhow::{bail, Context};

use crate::{
    config::Cache,
    huffman::{huffman_decode, huffman_encode},
    projects::{Project, ProjectLanguage},
    utils::GitInfo,
};

const MAGIC: &[u8; 4] = b"YMIR";
const VERSION: u8 = 4;

pub trait CacheSerializer {
    fn serialize(&self) -> anyhow::Result<Vec<u8>>;
    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl CacheSerializer for Cache {
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        // Projects len
        buffer.extend_from_slice(&u16::try_from(self.projects.len())?.to_le_bytes());

        for project in &self.projects {
            buffer.extend_from_slice(&project.serialize()?);
        }

        // Huffman encoding
        let mut new_buffer: Vec<u8> = Vec::new();
        new_buffer.extend_from_slice(MAGIC);
        new_buffer.push(VERSION);
        new_buffer.extend_from_slice(&huffman_encode(&buffer));

        Ok(new_buffer)
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
            bail!("Invalid version. Found: {}, current {VERSION}", version[0]);
        }

        // Huffman decoding
        let buffer = huffman_decode(&cursor.clone().into_inner()[cursor.position() as usize..])?;
        let mut cursor = std::io::Cursor::new(buffer.as_slice());

        let mut len_bytes = [0u8; 2];
        cursor
            .read_exact(&mut len_bytes)
            .with_context(|| "Failed to read projects length")?;
        let projects_len = u16::from_le_bytes(len_bytes) as usize;

        let mut projects: Vec<Project> = Vec::new();

        for _ in 0..projects_len {
            projects.push(Project::deserialize(&mut cursor)?);
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
        let languages: HashMap<u8, ProjectLanguage> = HashMap::deserialize(cursor)?;
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
        if let Some(remote_url) = &self.remote_url {
            buffer.extend_from_slice(&u16::try_from(remote_url.len())?.to_le_bytes());
            buffer.extend_from_slice(remote_url.as_bytes());
        } else {
            buffer.extend_from_slice(&0_u16.to_le_bytes());
        }

        // Init date
        buffer.extend_from_slice(&self.init_date.to_le_bytes());

        // Last commit date
        buffer.extend_from_slice(&self.last_commit_date.to_le_bytes());

        // Last commit msg
        if let Some(last_commit_msg) = &self.last_commit_msg {
            buffer.extend_from_slice(&u16::try_from(last_commit_msg.len())?.to_le_bytes());
            buffer.extend_from_slice(last_commit_msg.as_bytes());
        } else {
            buffer.extend_from_slice(&0_u16.to_le_bytes());
        }

        // Commit count
        buffer.extend_from_slice(&self.commit_count.to_le_bytes());

        Ok(buffer)
    }

    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let mut len_bytes = [0u8; 2];

        // Remote url
        cursor.read_exact(&mut len_bytes)?;
        let remote_url_len = u16::from_le_bytes(len_bytes) as usize;
        let remote_url: Option<String> = if remote_url_len > 0 {
            let mut remote_url = vec![0u8; remote_url_len];
            cursor
                .read_exact(&mut remote_url)
                .with_context(|| "Failed to read remote url")?;
            Some(String::from_utf8(remote_url).with_context(|| "Invalid UTF-8 key")?)
        } else {
            None
        };

        let mut init_date = u32::MAX.to_le_bytes();
        cursor.read_exact(&mut init_date)?;
        let init_date = u32::from_le_bytes(init_date);

        let mut last_commit_date = u32::MAX.to_le_bytes();
        cursor.read_exact(&mut last_commit_date)?;
        let last_commit_date = u32::from_le_bytes(last_commit_date);

        // Last commit msg
        cursor.read_exact(&mut len_bytes)?;
        let last_commit_msg_len = u16::from_le_bytes(len_bytes) as usize;
        let last_commit_msg = if last_commit_msg_len > 0 {
            let mut last_commit_msg = vec![0u8; last_commit_msg_len];
            cursor
                .read_exact(&mut last_commit_msg)
                .with_context(|| "Failed to read last commit msg")?;
            Some(String::from_utf8(last_commit_msg).with_context(|| "Invalid UTF-8 key")?)
        } else {
            None
        };

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

impl<T> CacheSerializer for HashMap<u8, T>
where
    T: CacheSerializer,
{
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.extend_from_slice(&u16::try_from(self.len())?.to_le_bytes());
        for (key, value) in self {
            // Key
            buffer.extend_from_slice(&key.to_le_bytes());

            // Value
            buffer.extend_from_slice(&T::serialize(value)?);
        }

        Ok(buffer)
    }

    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        // Hashmap count
        let mut hashmap_len = u16::MAX.to_le_bytes();
        cursor
            .read_exact(&mut hashmap_len)
            .with_context(|| "Failed to read hashmap len")?;
        let hashmap_len = u16::from_le_bytes(hashmap_len);

        let mut hashmap = HashMap::new();

        for _ in 0..hashmap_len {
            // Key
            let mut key = u8::MAX.to_le_bytes();
            cursor.read_exact(&mut key)?;
            let key = u8::from_le_bytes(key);

            // Value
            let value = T::deserialize(cursor)?;

            hashmap.insert(key, value);
        }

        Ok(hashmap)
    }
}
