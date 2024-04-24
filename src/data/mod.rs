use serde::Deserialize;
use serde::Serialize;
use sled::IVec;

/// A type that describes a page on the internet that we want to index.
#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub url: url::Url,
    pub title: String,
    pub content: String,
}

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

impl From<SearchResult> for IVec {
    fn from(val: SearchResult) -> Self {
        serde_json::to_string(&val).unwrap().as_bytes().into()
    }
}
