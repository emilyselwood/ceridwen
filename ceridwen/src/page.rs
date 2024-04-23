use serde::Deserialize;
use serde::Serialize;

/// A type that describes a page on the internet that we want to index.
#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub url: url::Url,
    pub title: String,
    pub content: String,
}
