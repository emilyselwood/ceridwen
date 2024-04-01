use log::info;
use std::path::Path;
use std::path::PathBuf;

use crate::data_file;
use crate::error::Error;

pub async fn lookup_word(root_dir: &str, word: &str) -> Result<Vec<WordIndexEntry>, Error> {
    let file_path = word_to_path(root_dir, word);

    read_word_index_file(file_path).await
}

pub async fn read_word_index_file(path: PathBuf) -> Result<Vec<WordIndexEntry>, Error> {
    info!("reading index file {:?}", path);
    if !path.exists() {
        // if the index file doesn't exist return an empty list as there isn't something with that word
        return Ok(Vec::new());
    }

    data_file::read_all(&path)
        .await?
        .iter()
        .map(|i| WordIndexEntry::from_string(i))
        .collect()
}

pub async fn append_word_index(path: PathBuf, entry: WordIndexEntry) -> Result<(), Error> {
    data_file::append(&path, &entry.to_record()).await?;
    Ok(())
}

pub async fn delete_url(path: PathBuf, url: String) -> Result<(), Error> {
    data_file::find_and_delete(&path, &url).await
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
    pub fn from_string(line: &str) -> Result<WordIndexEntry, Error> {
        let parts: Vec<&str> = line.split("::").collect();
        if parts.len() != 2 {
            return Err(Error::BadIndexRecord);
        }

        let count = parts[1].parse::<usize>()?;
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
