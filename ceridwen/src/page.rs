/// A type that describes a page on the internet that we want to index.
pub struct Page {
    pub url: url::Url,
    pub title: String,
    pub content: String,
}
