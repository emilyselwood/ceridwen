use chrono::Utc;
use log::{debug, info};

use crate::{
    config::Config,
    data::{Page, SearchResult},
    error::Error,
    utils::text_tools::{count_words, filter, tokenise},
};

pub mod postgres;

pub trait Index<ID> {
    fn connect(config: &Config) -> Result<Self, Error>
    where
        Self: std::marker::Sized;

    fn search(&mut self, search_string: &str) -> Result<Vec<SearchResult>, Error>;

    fn look_up_page(&mut self, page: &Page) -> Result<Option<(ID, SearchResult)>, Error>;

    fn lookup_id(&mut self, id: ID) -> Result<Option<SearchResult>, Error>;

    fn store_page(&mut self, page: &Page) -> Result<(ID, SearchResult), Error>;

    fn store_words(&mut self, page_id: ID, words: Vec<(String, u64)>) -> Result<(), Error>;

    fn add_page(
        &mut self,
        page: &Page,
        min_update_interval: &chrono::Duration,
    ) -> impl std::future::Future<Output = Result<(), Error>> {
        async {
            // check if we have the page already, and if its old enough to need an update
            let existing_result = self.look_up_page(page)?;

            let (page_id, _page_result) = if let Some((id, search_result)) = existing_result {
                if search_result.last_index + *min_update_interval > Utc::now() {
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
    }
}
