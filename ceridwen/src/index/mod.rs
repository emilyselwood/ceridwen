use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use chrono::Utc;
use log::debug;
use log::info;
use serde::Deserialize;
use serde::Serialize;

use crate::error::Error;
use crate::index::word_index::WordIndexEntry;
use crate::page::Page;
use crate::system_root;

mod page_index;
mod search;
mod word_index;

pub fn index_dir() -> PathBuf {
    system_root().join("index")
}

// TODO: Locking of files.

#[derive(Debug)]
pub struct Index {
    root_dir: String,
}

impl Index {
    pub fn new(root_dir: &PathBuf) -> Result<Self, Error> {
        info!("Creating index at {root_dir:?}");
        let root_path = Path::new(&root_dir);
        // Make sure the folder doesn't exist.
        if root_path.exists() {
            return Err(Error::IndexDirAlreadyExists(
                root_path.to_str().unwrap().to_string(),
            ));
        }

        // Create the folders
        fs::create_dir_all(root_path)?;

        Ok(Index {
            root_dir: root_dir.to_str().unwrap().to_string(),
        })
    }

    pub fn load(root_dir: &PathBuf) -> Result<Self, Error> {
        info!("Loading index at {:?}", root_dir);
        let root_path = Path::new(&root_dir);
        // check the path exists
        if !root_path.exists() {
            return Err(Error::IndexDirDoesNotExist(
                root_path.to_str().unwrap().to_string(),
            ));
        }

        Ok(Index {
            root_dir: root_path.to_str().unwrap().to_string(),
        })
    }

    pub fn search(&self, search_string: &str) -> Result<Vec<SearchResult>, Error> {
        search::search(&self.root_dir, search_string)
    }

    pub fn add_page(&mut self, page: &Page) -> Result<(), Error> {
        info!("adding {} to index", page.url);
        let mut words = tokenise(&page.title);
        words.append(&mut tokenise(&page.content));

        debug!("found {} tokens for {}", words.len(), page.url);
        words = filter(words);
        debug!("filtered to {} tokens for {}", words.len(), page.url);

        let word_counts = count_words(words);

        for (word, count) in word_counts.iter() {
            debug!(
                "adding {} to index with count {} for {}",
                word, count, page.url
            );
            self.add_to_word_index(page, word, *count)?;
        }
        debug!("adding {} to page index", page.url);
        self.add_to_page_index(page, &word_counts)?;

        debug!("finished adding {} to the index", page.url);
        Ok(())
    }

    pub fn delete_page(&mut self, _url: String) -> Result<(), Error> {
        // get the list of words for a url
        // go through and remove the page from every one of those words.

        todo!()
    }

    fn add_to_word_index(&mut self, page: &Page, word: &str, count: usize) -> Result<(), Error> {
        let index_file = word_index::word_to_path(&self.root_dir, word);

        word_index::append_word_index(
            index_file,
            WordIndexEntry::new(page.url.to_string(), count),
        )?;

        Ok(())
    }

    fn add_to_page_index(
        &mut self,
        page: &Page,
        words: &Vec<(String, usize)>,
    ) -> Result<(), Error> {
        let index_file_path = page_index::url_to_words_path(&self.root_dir, &page.url);
        page_index::write_page_words(index_file_path, words)?;

        let page_file_path = page_index::url_to_page_path(&self.root_dir, &page.url);
        page_index::write_page_details(page_file_path, &page.into())?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
    pub last_index: chrono::DateTime<Utc>,
}

impl From<&Page> for SearchResult {
    fn from(value: &Page) -> Self {
        SearchResult {
            url: value.url.to_string(),
            title: value.title.clone(),
            description: value.content.chars().take(250).collect(),
            last_index: Utc::now(),
        }
    }
}

fn tokenise(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(str::to_lowercase)
        .map(|w| {
            w.trim().replace(
                &[
                    '(', ')', ',', '\"', '.', ';', ':', '\'', '?', '<', '>', '\\', '/',
                ][..],
                "",
            )
        })
        .collect()
}

fn filter(words: Vec<String>) -> Vec<String> {
    // TODO: implement stop word filters
    words
}

fn count_words(words: Vec<String>) -> Vec<(String, usize)> {
    let mut result: HashMap<&String, usize> = HashMap::new();

    for word in words.iter() {
        *result.entry(word).or_insert(0) += 1;
    }

    result
        .iter()
        .map(|(k, v)| ((*k).clone(), v.clone()))
        .collect()
}