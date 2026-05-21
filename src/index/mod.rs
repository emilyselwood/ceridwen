use log::{debug, info};
use sled::IVec;

use crate::{
    config::Config,
    data::{Page, SearchResult},
    error::Error,
    utils::text_tools::{count_words, filter, tokenise},
};

pub trait Index {
    fn connect(config: &Config) -> Self;

    fn search(&self, search_string: &str) -> Result<Vec<SearchResult>, Error>;

    fn last_index_time(
        &self,
        page: &Page,
    ) -> impl std::future::Future<Output = Result<Option<time::OffsetDateTime>, Error>>;

    fn look_up_page(&self, page: &Page) -> Result<Option<(IVec, SearchResult)>, Error>;

    fn lookup_id(&self, id: u64) -> Result<Option<SearchResult>, Error>;

    fn store_page(&self, page: &Page) -> Result<(IVec, SearchResult), Error>;

    fn store_words(&self, page_id: IVec, words: Vec<(String, u64)>) -> Result<(), Error>;

    fn add_page(
        &self,
        page: &Page,
        min_update_interval: &time::Duration,
    ) -> impl std::future::Future<Output = Result<(), Error>> {
        async {
            // check if we have the page already, and if its old enough to need an update
            let existing_result = self.look_up_page(page)?;

            let (page_id, _page_result) = if let Some((id, search_result)) = existing_result {
                if search_result.last_index + *min_update_interval > time::OffsetDateTime::now_utc()
                {
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
