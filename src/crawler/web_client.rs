use std::path::Path;
use std::time::Duration;

use crate::config::Config;
use crate::error::Error;
use crate::utils::percentage::percentage;

use bytes::Bytes;
use humansize::{format_size, DECIMAL};
use log::debug;
use log::warn;
use reqwest::Client;
use reqwest::StatusCode;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use url::Url;

pub const USER_AGENT: &str = "ceridwen-crawler";

pub fn get_client(_config: &Config) -> Result<Client, Error> {
    Ok(Client::builder()
        .user_agent(USER_AGENT)
        .tcp_keepalive(Duration::new(5, 0))
        .build()?)
}

pub async fn get(client: &Client, url: &str) -> Result<Bytes, Error> {
    debug!("Making request for {}", url);
    let start_time = time::Instant::now();

    let response = client.execute(client.get(url).build()?).await?;

    debug!("Got {} back from {}", response.status(), url);
    if response.status() == StatusCode::NOT_FOUND {
        return Err(Error::PageNotFound(url.to_string()));
    } else if !response.status().is_success() {
        return Err(Error::Request(response.status()));
    }

    let file_bytes = response.bytes().await?;
    debug!(
        "Response size: {} for {} in {}",
        file_bytes.len(),
        url,
        start_time.elapsed()
    );

    Ok(file_bytes)
}

pub async fn get_url(client: &Client, url: &Url) -> Result<Bytes, Error> {
    get(client, url.as_str()).await
}

pub async fn get_to_file(client: &Client, url: &str, target_path: &Path) -> Result<(), Error> {
    debug!("Attempting to download {url} to {target_path:?}");
    fs::create_dir_all(target_path.parent().unwrap()).await?;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(target_path)
        .await?;

    let download_start = time::Instant::now();

    let mut response = client.get(url).send().await?;

    debug!("Got {} back from {}", response.status(), url);
    if response.status() == StatusCode::NOT_FOUND {
        return Err(Error::PageNotFound(url.to_string()));
    } else if !response.status().is_success() {
        return Err(Error::Request(response.status()));
    }

    let total_size = response.content_length();
    if let Some(content_length) = total_size {
        debug!("got content length of {} for {}", content_length, url)
    }

    let mut downloaded: usize = 0;
    let mut last_log_size = 0;
    while let Some(chunk) = response.chunk().await? {
        let amount = file.write(&chunk[..]).await?;
        if amount != chunk.len() {
            warn!("Incomplete write of download chunk! Aborting. {}", url);
            return Err(Error::IncompleteWrite(
                target_path.to_str().unwrap().to_string(),
                (downloaded + amount) as u64,
            ));
        }

        downloaded += chunk.len();

        // log every 1% if we have a content length or every 100mb otherwise
        if let Some(content_length) = total_size {
            if percentage(content_length, (downloaded - last_log_size) as u64) > 1.0 {
                debug!(
                    "Downloaded {:.02}% ({}/{}) of {}",
                    percentage(content_length, downloaded as u64),
                    format_size(downloaded, DECIMAL),
                    format_size(content_length, DECIMAL),
                    url
                );
                last_log_size = downloaded;
            }
        } else if downloaded - last_log_size > 100_000_000 {
            debug!("Downloaded {} of {}", format_size(downloaded, DECIMAL), url);
            last_log_size = downloaded;
        }
    }

    let download_duration = download_start.elapsed();
    debug!("Download took: {}", download_duration);

    Ok(())
}
