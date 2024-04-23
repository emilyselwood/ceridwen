use serde::Deserialize;
use serde::Serialize;
use sled::IVec;

use crate::page::Page;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
    pub last_index: time::OffsetDateTime,
}

impl From<&Page> for SearchResult {
    fn from(value: &Page) -> Self {
        SearchResult {
            url: value.url.to_string(),
            title: value.title.clone(),
            description: value.content.chars().take(250).collect(),
            last_index: time::OffsetDateTime::now_utc(),
        }
    }
}

impl From<IVec> for SearchResult {
    fn from(value: IVec) -> Self {
        serde_json::from_slice(value.as_ref()).unwrap()
    }
}

impl Into<IVec> for SearchResult {
    fn into(self) -> IVec {
        serde_json::to_string(&self).unwrap().as_bytes().into()
    }
}
