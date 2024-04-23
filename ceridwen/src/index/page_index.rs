use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::OpenOptions;

use log::debug;
use log::warn;
use sha3::Digest;
use sha3::Sha3_256;
use tokio::io::AsyncWriteExt;

use crate::error::Error;
use crate::index::SearchResult;

/* Write the entire list of words to disk for this page. All at once, no appending. */
pub async fn write_page_words(file_path: PathBuf, words: &[(String, u64)]) -> Result<(), Error> {
    if file_path.exists() {
        warn!("Index file {:?} already exists. Overwriting it.", file_path)
    }
    fs::create_dir_all(file_path.parent().unwrap()).await?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(file_path)
        .await?;

    for (word, count) in words.iter() {
        file.write_all(format!("{}::{}\n", word, count).as_bytes())
            .await?;
    }

    Ok(())
}

/* Load a pages word list. */
pub async fn load_page_words(file_path: PathBuf) -> Result<Vec<(String, usize)>, Error> {
    if !file_path.exists() {
        return Err(Error::BadIndexRecord);
    }

    let mut result = Vec::new();

    let content = fs::read_to_string(file_path).await?;
    for line in content.lines() {
        let parts: Vec<&str> = line.split("::").collect();
        result.push((parts[0].to_owned(), parts[1].parse::<usize>()?))
    }

    Ok(result)
}

pub async fn load_page_details(file_path: PathBuf) -> Result<SearchResult, Error> {
    if !file_path.exists() {
        warn!("Expected page details file does not exist! {:?}", file_path);
        return Err(Error::BadIndexRecord);
    }

    debug!("loading page details: {:?}", file_path);

    let content = fs::read_to_string(file_path).await?;
    let result: SearchResult = toml::from_str(&content)?;

    Ok(result)
}

pub async fn write_page_details(
    file_path: PathBuf,
    page_details: &SearchResult,
) -> Result<(), Error> {
    let content = toml::to_string_pretty(page_details)?;

    tokio::fs::write(file_path, content).await?;

    Ok(())
}

pub fn url_to_words_path(root_path: &str, url: &url::Url) -> PathBuf {
    let mut result = url_to_path(root_path, url);
    result.set_extension("words");
    result
}

pub fn url_to_page_path(root_path: &str, url: &url::Url) -> PathBuf {
    let mut result = url_to_path(root_path, url);
    result.set_extension("page");
    result
}

pub fn url_to_path(root_path: &str, url: &url::Url) -> PathBuf {
    let domain = url.host().unwrap().to_string();

    // Split things down by reverse domain
    let mut domain_parts: Vec<&str> = domain.split('.').collect();
    domain_parts.reverse();

    let mut result = Path::new(root_path).join("pages");

    for part in domain_parts.iter() {
        result = result.join(part);
    }

    // now to handle the file section...
    for segment in url.path_segments().unwrap() {
        result = result.join(segment);
    }

    // last we create a file from the hash of the rest of the url,
    let mut hasher = Sha3_256::new();

    hasher.update(url.query().unwrap_or("no query").as_bytes());
    hasher.update(url.fragment().unwrap_or("no fragment").as_bytes());

    let hash = hasher.finalize();
    result = result.join(hex::encode(hash));
    result
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use url::Url;

    use crate::index::page_index::url_to_words_path;
    use crate::normalise_path;

    #[test]
    fn tests_url_to_path() {
        let test_url = Url::parse("https://example.com/foo/bar/index.html?q=something").unwrap();

        let result = url_to_words_path("/tmp/ceridwen", &test_url);

        let expected = normalise_path(
            PathBuf::from("/tmp/ceridwen/pages/com/example/foo/bar/index.html/edd38c9673866b98b2397ee88345fdbe3d1177e5a2ba71b97ad1f71ce40443e6.words")
        );

        assert_eq!(normalise_path(result), expected)
    }
}
