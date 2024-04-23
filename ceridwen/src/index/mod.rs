use std::fs;
use std::path::Path;
use std::path::PathBuf;

use log::debug;
use log::info;

use crate::error::Error;
use crate::index::word_index::WordIndexEntry;
use crate::page::Page;
use crate::search_result::SearchResult;
use crate::system_root;
use crate::text_tools::count_words;
use crate::text_tools::filter;
use crate::text_tools::tokenise;

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

    pub async fn search(&self, search_string: &str) -> Result<Vec<SearchResult>, Error> {
        search::search(&self.root_dir, search_string).await
    }

    pub async fn add_page(
        &mut self,
        page: &Page,
        min_update_interval: time::Duration,
    ) -> Result<(), Error> {
        // check if we have the page already, and if its old enough to need an update
        if let Some(last_index_time) = self.last_index_time(page).await? {
            if last_index_time + min_update_interval > time::OffsetDateTime::now_utc() {
                info!(
                    "Last indexed {} at {} its too soon to do it again.",
                    page.url, last_index_time
                );
                return Ok(());
            } else {
                info!("{} already exists in index", page.url);
                // delete the existing records so we can update them.
                self.delete_page(&page.url).await?;
            }
        }
        info!("adding {} to word index", page.url);
        let mut words = tokenise(&page.title);
        words.append(&mut tokenise(&page.content));

        debug!("found {} tokens for {}", words.len(), page.url);
        words = filter(words);
        debug!("filtered to {} tokens for {}", words.len(), page.url);

        let word_counts = count_words(words);

        for (word, count) in word_counts.iter() {
            // debug!(
            //     "adding {} to index with count {} for {}",
            //     word, count, page.url
            // );
            self.add_to_word_index(page, word, *count).await?;
        }
        debug!("adding {} to page index", page.url);
        self.add_to_page_index(page, &word_counts).await?;

        debug!("finished adding {} to the index", page.url);
        Ok(())
    }

    pub async fn delete_page(&mut self, url: &url::Url) -> Result<(), Error> {
        info!("deleting {} from index", url);

        // get the list of words for a url
        let word_file_path = page_index::url_to_words_path(&self.root_dir, url);
        let words = page_index::load_page_words(word_file_path).await?;

        // go through and remove the page from every one of those words ... this is going to be evil.

        for (word, _count) in words.iter() {
            let word_path = word_index::word_to_path(&self.root_dir, word);
            word_index::delete_url(word_path, url.to_string()).await?;
        }

        Ok(())
    }

    async fn add_to_word_index(
        &mut self,
        page: &Page,
        word: &str,
        count: u64,
    ) -> Result<(), Error> {
        let index_file = word_index::word_to_path(&self.root_dir, word);

        word_index::append_word_index(index_file, WordIndexEntry::new(page.url.to_string(), count))
            .await?;

        Ok(())
    }

    async fn add_to_page_index(
        &mut self,
        page: &Page,
        words: &[(String, u64)],
    ) -> Result<(), Error> {
        let index_file_path = page_index::url_to_words_path(&self.root_dir, &page.url);
        page_index::write_page_words(index_file_path, words).await?;

        let page_file_path = page_index::url_to_page_path(&self.root_dir, &page.url);
        page_index::write_page_details(page_file_path, &page.into()).await?;

        Ok(())
    }

    async fn last_index_time(&self, page: &Page) -> Result<Option<time::OffsetDateTime>, Error> {
        let page_file_path = page_index::url_to_page_path(&self.root_dir, &page.url);
        if !page_file_path.exists() {
            return Ok(None);
        }
        let page_file = page_index::load_page_details(page_file_path).await?;

        Ok(Some(page_file.last_index))
    }
}
