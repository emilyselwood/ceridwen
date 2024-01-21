use std::time::Duration;

use bytes::Bytes;
use ceridwen::config::Config;
use ceridwen::error::Error;
use log::debug;
use reqwest::Client;
use reqwest::StatusCode;
use url::Url;

pub const USER_AGENT: &str = "ceridwen-crawler";

pub fn get_client(_config: &Config) -> Result<Client, Error> {
    Ok(Client::builder()
        .user_agent(USER_AGENT)
        .tcp_keepalive(Duration::new(5, 0))
        .build()?)
}

pub async fn get_url(client: &Client, url: &Url) -> Result<Bytes, Error> {
    debug!("Making request for {}", url);

    let response = client.execute(client.get(url.as_str()).build()?).await?;

    debug!("Got {} back from {}", response.status(), url);
    if response.status() == StatusCode::NOT_FOUND {
        return Err(Error::PageNotFound(url.to_string()));
    }

    let file_bytes = response.bytes().await?;
    debug!("Response size: {} for {}", file_bytes.len(), url);

    Ok(file_bytes)
}
