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

        let projects_len = cursor
            .read_u16()
            .with_context(|| "Failed to read projects_len")? as usize;

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

        let path = self.path.to_string_lossy();
        buffer.extend_from_slice(&u16::try_from(path.len())?.to_le_bytes());
        buffer.extend_from_slice(&path.to_string().as_bytes());

        buffer.extend_from_slice(&self.size.to_le_bytes());

        buffer.extend_from_slice(&GitInfo::serialize(&self.git_info)?);

        buffer.extend_from_slice(&self.languages.serialize()?);
        buffer.extend_from_slice(&ProjectLanguage::serialize(&self.languages_total)?);

        Ok(buffer)
    }

    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let path_len = cursor
            .read_u16()
            .with_context(|| "Failed to read path len")? as usize;

        let path = cursor
            .read_string(path_len)
            .with_context(|| "Failed to read path")?;
        let path = PathBuf::from(path);

        let size = cursor.read_u64().with_context(|| "Failed to read size")?;

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

        if let Some(remote_url) = &self.remote_url {
            buffer.extend_from_slice(&u16::try_from(remote_url.len())?.to_le_bytes());
            buffer.extend_from_slice(remote_url.as_bytes());
        } else {
            buffer.extend_from_slice(&0_u16.to_le_bytes());
        }

        buffer.extend_from_slice(&self.init_date.to_le_bytes());
        buffer.extend_from_slice(&self.last_commit_date.to_le_bytes());

        if let Some(last_commit_msg) = &self.last_commit_msg {
            buffer.extend_from_slice(&u16::try_from(last_commit_msg.len())?.to_le_bytes());
            buffer.extend_from_slice(last_commit_msg.as_bytes());
        } else {
            buffer.extend_from_slice(&0_u16.to_le_bytes());
        }

        buffer.extend_from_slice(&self.commit_count.to_le_bytes());

        Ok(buffer)
    }

    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let remote_url_len = cursor
            .read_u16()
            .with_context(|| "Failed to read remote url len")?;

        let remote_url = if remote_url_len > 0 {
            cursor
                .read_string(remote_url_len as usize)
                .with_context(|| "Failed to read remote url")
                .ok()
        } else {
            None
        };

        let init_date = cursor
            .read_u32()
            .with_context(|| "Failed to read init date")?;

        let last_commit_date = cursor
            .read_u32()
            .with_context(|| "Failed to read last commit date")?;

        let last_commit_msg_len = cursor
            .read_u16()
            .with_context(|| "Failed to read last commit msg len")?;

        let last_commit_msg = if last_commit_msg_len > 0 {
            cursor
                .read_string(last_commit_msg_len as usize)
                .with_context(|| "Failed to read last commit msg")
                .ok()
        } else {
            None
        };

        let commit_count = cursor
            .read_u32()
            .with_context(|| "Failed to read commit count")?;

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
        let files = cursor.read_u32().with_context(|| "Failed to read files")?;
        let lines = cursor.read_u32().with_context(|| "Failed to read lines")?;
        let code = cursor.read_u32().with_context(|| "Failed to read code")?;
        let comments = cursor
            .read_u32()
            .with_context(|| "Failed to read comments")?;
        let blanks = cursor.read_u32().with_context(|| "Failed to read blanks")?;

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
            buffer.extend_from_slice(&key.to_le_bytes());
            buffer.extend_from_slice(&T::serialize(value)?);
        }

        Ok(buffer)
    }

    fn deserialize(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
        let hashmap_len = cursor
            .read_u16()
            .with_context(|| "Failed to read hashmap len")?;

        let mut hashmap = HashMap::new();

        for _ in 0..hashmap_len {
            let key = cursor.read_u8().with_context(|| "Failed to read key")?;
            let value = T::deserialize(cursor)?;

            hashmap.insert(key, value);
        }

        Ok(hashmap)
    }
}

pub trait CursorUtil {
    fn read_u8(&mut self) -> anyhow::Result<u8>;
    fn read_u16(&mut self) -> anyhow::Result<u16>;
    fn read_u32(&mut self) -> anyhow::Result<u32>;
    fn read_u64(&mut self) -> anyhow::Result<u64>;
    fn read_string(&mut self, len: usize) -> anyhow::Result<String>;
}

impl CursorUtil for Cursor<&[u8]> {
    fn read_u8(&mut self) -> anyhow::Result<u8> {
        let mut bytes = [0u8; 1];
        self.read_exact(&mut bytes)?;
        Ok(u8::from_le_bytes(bytes))
    }

    fn read_u16(&mut self) -> anyhow::Result<u16> {
        let mut bytes = [0u8; 2];
        self.read_exact(&mut bytes)?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_u32(&mut self) -> anyhow::Result<u32> {
        let mut bytes = [0u8; 4];
        self.read_exact(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_u64(&mut self) -> anyhow::Result<u64> {
        let mut bytes = [0u8; 8];
        self.read_exact(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn read_string(&mut self, len: usize) -> anyhow::Result<String> {
        let mut bytes = vec![0u8; len];
        self.read_exact(&mut bytes)?;
        Ok(String::from_utf8(bytes).with_context(|| "Invalid UTF-8 key")?)
    }
}
