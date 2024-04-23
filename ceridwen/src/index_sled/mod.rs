use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use log::debug;
use log::info;
use log::warn;
use sled::IVec;

use crate::error::Error;
use crate::page::Page;
use crate::search_result::SearchResult;
use crate::system_root;
use crate::text_tools::count_words;
use crate::text_tools::filter;
use crate::text_tools::tokenise;

pub fn index_path() -> PathBuf {
    system_root().join("index")
}

fn word_db() -> &'static sled::Db {
    static DB: OnceLock<sled::Db> = OnceLock::new();
    DB.get_or_init(|| sled::open(index_path().join("word_index")).unwrap())
}

const WORD_KEY_SEPARATOR: u8 = '=' as u8;

fn page_db() -> &'static sled::Db {
    static DB: OnceLock<sled::Db> = OnceLock::new();
    DB.get_or_init(|| sled::open(index_path().join("page_index")).unwrap())
}

fn page_url_db() -> &'static sled::Db {
    static DB: OnceLock<sled::Db> = OnceLock::new();
    DB.get_or_init(|| sled::open(index_path().join("page_url_index")).unwrap())
}

#[derive(Debug, Clone)]
pub struct Index {}

impl Index {
    pub async fn load() -> Result<Self, Error> {
        Ok(Index {})
    }

    pub async fn search(&self, search_string: &str) -> Result<Vec<SearchResult>, Error> {
        let words = tokenise(search_string);
        info!("Searching for matches to: {words:?}");
        let mut possible_pages: HashMap<u64, u64> = HashMap::new();
        for word in words.into_iter() {
            let mut word_bytes = word.as_bytes().to_vec();
            word_bytes.push(WORD_KEY_SEPARATOR);
            let num_bytes = word_bytes.len();
            info!("scanning for {}=", word);
            for row in word_db().scan_prefix(word_bytes) {
                if let Ok((key, value)) = row {
                    let id = u64::from_be_bytes(
                        key[num_bytes..]
                            .try_into()
                            .expect(&format!("could not decode key {:?}", key)),
                    );
                    let count = u64::from_be_bytes(value[..].try_into().unwrap());

                    *possible_pages.entry(id).or_insert(0) += count;
                } else {
                    return Err(row.unwrap_err().into());
                }
            }
        }
        info!("Found {} possible pages", possible_pages.len());

        let mut frequencies = possible_pages.iter().collect::<Vec<_>>();
        frequencies.sort_unstable_by_key(|e| e.1);
        frequencies.reverse();
        frequencies.truncate(100);

        info!(
            "after sorting and truncation got {} pages",
            frequencies.len()
        );

        let mut result = Vec::new();
        for (id, _count) in frequencies.into_iter() {
            let search_result = self.lookup_id(id.clone())?;
            match search_result {
                Some(r) => result.push(r),
                None => warn!(
                    "Search result returned id {} which doesn't have a page entry. Index is broke!",
                    id
                ),
            }
        }
        info!("finished looking up details for {} pages", result.len());
        Ok(result)
    }

    pub async fn add_page(
        &self,
        page: &Page,
        min_update_interval: time::Duration,
    ) -> Result<(), Error> {
        // check if we have the page already, and if its old enough to need an update
        let existing_result = self.look_up_page(page)?;

        let (page_id, _page_result) = if let Some((id, search_result)) = existing_result {
            if search_result.last_index + min_update_interval > time::OffsetDateTime::now_utc() {
                info!(
                    "Last indexed {} at {} its too soon to do it again.",
                    page.url, search_result.last_index
                );
                return Ok(());
            } else {
                // TODO: clear existing words for this page?
            }

            (id, search_result)
        } else {
            self.store_page(page)?
        };

        info!("adding {} to word index", page.url);
        let mut words = tokenise(&page.title);
        words.append(&mut tokenise(&page.content));

        // debug!("found {} tokens for {}", words.len(), page.url);
        words = filter(words);
        debug!("filtered to {} tokens for {}", words.len(), page.url);

        let word_counts = count_words(words);

        self.store_words(page_id, word_counts)
    }

    pub async fn last_index_time(
        &self,
        page: &Page,
    ) -> Result<Option<time::OffsetDateTime>, Error> {
        let url = page.url.to_string();

        let page_id = page_url_db().get(url.bytes().collect::<Vec<_>>())?;
        if page_id.is_none() {
            return Ok(None);
        }

        Ok(page_db()
            .get(page_id.unwrap())?
            .map(|b| SearchResult::from(b))
            .map(|s| s.last_index))
    }

    pub fn look_up_page(&self, page: &Page) -> Result<Option<(IVec, SearchResult)>, Error> {
        let url = page.url.to_string();

        let page_id = page_url_db().get(url.bytes().collect::<Vec<_>>())?;
        if page_id.is_none() {
            return Ok(None);
        }
        let page_id = page_id.unwrap();

        let page = page_db().get(&page_id)?.map(|b| SearchResult::from(b));

        if page.is_none() {
            return Err(Error::BadIndexRecord);
        }

        Ok(Some((page_id, page.unwrap())))
    }

    pub fn lookup_id(&self, id: u64) -> Result<Option<SearchResult>, Error> {
        let page: Option<SearchResult> = page_db()
            .get(id.to_be_bytes())?
            .map(|b| SearchResult::from(b));
        Ok(page)
    }

    pub fn store_page(&self, page: &Page) -> Result<(IVec, SearchResult), Error> {
        let search_result: SearchResult = page.into();
        let id: IVec = (&page_db().generate_id()?.to_be_bytes()).into();

        page_url_db().insert(page.url.to_string().bytes().collect::<Vec<_>>(), id.clone())?;

        let page_data: IVec = search_result.clone().into();
        page_db().insert(id.clone(), page_data)?;

        Ok((id, search_result))
    }

    fn store_words(&self, page_id: IVec, words: Vec<(String, u64)>) -> Result<(), Error> {
        let mut batch = sled::Batch::default();
        for (word, count) in words.into_iter() {
            let mut key: Vec<u8> = word.as_bytes().to_vec();
            key.push(WORD_KEY_SEPARATOR);
            key.append(&mut page_id.as_ref().to_vec());
            let value: IVec = (&count.to_be_bytes()).into();
            batch.insert(key, value);
        }
        word_db().apply_batch(batch)?;
        Ok(())
    }
}
