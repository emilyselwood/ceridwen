use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

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
