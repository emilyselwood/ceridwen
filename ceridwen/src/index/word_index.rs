use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

use log::info;

use crate::error::Error;
use crate::read_lines;

pub fn lookup_word(root_dir: &str, word: &str) -> Result<Vec<WordIndexEntry>, Error> {
    let file_path = word_to_path(root_dir, word);

    read_word_index_file(file_path)
}

pub fn read_word_index_file(path: PathBuf) -> Result<Vec<WordIndexEntry>, Error> {
    info!("reading index file {:?}", path);
    let mut result = Vec::new();
    let lines = read_lines(path.clone())?;
    // Consumes the iterator, returns an (Optional) String
    for line in lines {
        result.push(WordIndexEntry::from_string(line.unwrap())?);
    }
    info!("read index file {:?}, got {} entries", path, result.len());
    Ok(result)
}

pub fn append_word_index(path: PathBuf, entry: WordIndexEntry) -> Result<(), Error> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    writeln!(file, "{}", entry.to_record())?;

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
