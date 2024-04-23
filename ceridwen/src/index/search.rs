use std::collections::HashMap;

use log::debug;
use url::Url;

use crate::error::Error;
use crate::index::page_index;
use crate::index::tokenise;
use crate::index::word_index;
use crate::index::word_index::WordIndexEntry;
use crate::index::SearchResult;

pub async fn search(root_dir: &str, search_string: &str) -> Result<Vec<SearchResult>, Error> {
    // split the search string into words.
    let words = tokenise(search_string);

    // get a list of pages for all of those words
    let mut pages: Vec<WordIndexEntry> = Vec::new();
    for word in words.iter() {
        pages.append(&mut word_index::lookup_word(root_dir, word).await?);
    }

    // count number of instances of those urls
    let mut url_counts: HashMap<String, u64> = HashMap::new();
    for entry in pages.iter() {
        *url_counts.entry(entry.url.clone()).or_insert(0) += entry.count
    }

    // sort results highest counts go first and take the first 100
    let mut frequencies = url_counts.iter().collect::<Vec<_>>();
    frequencies.sort_unstable_by_key(|e| e.1);
    frequencies.reverse();
    frequencies.truncate(100);

    // convert into SearchResults
    let mut result = Vec::new();
    for (url_str, _count) in frequencies.iter() {
        let url = Url::parse(url_str)?;
        let file_path = page_index::url_to_page_path(root_dir, &url);
        result.push(page_index::load_page_details(file_path).await?);
    }
    debug!("found {} results", result.len());
    Ok(result)
}
