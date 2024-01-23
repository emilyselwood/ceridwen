use log::debug;
use log::info;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;

use crate::error::Error;

pub async fn lookup_word(root_dir: &str, word: &str) -> Result<Vec<WordIndexEntry>, Error> {
    let file_path = word_to_path(root_dir, word);

    read_word_index_file(file_path).await
}

pub async fn read_word_index_file(path: PathBuf) -> Result<Vec<WordIndexEntry>, Error> {
    info!("reading index file {:?}", path);
    let mut result = Vec::new();

    if !path.exists() {
        debug!("index file doesn't exist, this word is not in our index");
        return Ok(result);
    }

    let file = fs::OpenOptions::new()
        .read(true)
        .open(path.as_path())
        .await?;

    let buffered_reader = io::BufReader::new(file);
    let mut lines = buffered_reader.lines();
    // Consumes the iterator, returns an (Optional) String
    while let Some(line) = lines.next_line().await? {
        result.push(WordIndexEntry::from_string(line)?);
    }
    info!("read index file {:?}, got {} entries", path, result.len());
    Ok(result)
}

pub async fn append_word_index(path: PathBuf, entry: WordIndexEntry) -> Result<(), Error> {
    fs::create_dir_all(path.parent().unwrap()).await?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;

    file.write_all(format!("{}\n", entry.to_record()).as_bytes())
        .await?;

    Ok(())
}

pub fn word_to_path(root_dir: &str, word: &str) -> PathBuf {
    Path::new(root_dir)
        .join("words")
        .join(word.chars().next().unwrap().to_string())
        .join(word.len().to_string())
        .join(format!("{word}.index"))
}

pub struct WordIndexEntry {
    pub url: String,
    pub count: usize,
}

impl WordIndexEntry {
    pub fn from_string(line: String) -> Result<WordIndexEntry, Error> {
        let parts: Vec<&str> = line.split("::").collect();
        if parts.len() != 2 {
            return Err(Error::BadIndexRecord);
        }

        let count = usize::from_str_radix(parts[1], 10)?;
        Ok(WordIndexEntry {
            url: parts[0].to_string(),
            count,
        })
    }

    pub fn new(url: String, count: usize) -> Self {
        WordIndexEntry { url, count }
    }

    pub fn to_record(&self) -> String {
        format!("{}::{}", self.url, self.count)
    }
}
