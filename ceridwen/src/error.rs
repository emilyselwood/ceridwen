use std::num::ParseIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // index errors
    #[error("New index directory {0:?} already exists")]
    IndexDirAlreadyExists(String),
    #[error("Index directory {0:?} doesn't exist")]
    IndexDirDoesNotExist(String),
    #[error("Bad Index Record")]
    BadIndexRecord,

    // crawler errors
    #[error("Unknown ingester {0}")]
    UnknownIngester(String),
    #[error("Missing base url")]
    MissingBaseUrl,

    // url errors
    #[error("Missing host: {0}")]
    MissingHost(String),

    // Page loading errors
    #[error("Page not found (404): {0}")]
    PageNotFound(String),

    // Std lib errors
    #[error("Could not parse int: {0:?}")]
    ParseInt(ParseIntError),
    #[error("Could not parse url: {0:?}")]
    UrlParsing(url::ParseError),
    #[error("IO Error: {0:?}")]
    IOError(std::io::Error),

    // dependencies errors
    #[error("Could not join: {0:?}")]
    TokioJoin(tokio::task::JoinError),
    #[error("Toml Deserialize Error: {0:?}")]
    TomlDeserializeError(toml::de::Error),
    #[error("Toml Serialize Error: {0:?}")]
    TomlSerializeError(toml::ser::Error),
    #[error("Reqwest error: {0:?}")]
    ReqwestError(reqwest::Error),
    #[error("RSS error: {0:?}")]
    RSSError(rss::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::IndexDirAlreadyExists(l0), Self::IndexDirAlreadyExists(r0)) => l0 == r0,
            (Self::IndexDirDoesNotExist(l0), Self::IndexDirDoesNotExist(r0)) => l0 == r0,
            (Self::UnknownIngester(l0), Self::UnknownIngester(r0)) => l0 == r0,
            (Self::MissingHost(l0), Self::MissingHost(r0)) => l0 == r0,
            (Self::PageNotFound(l0), Self::PageNotFound(r0)) => l0 == r0,
            (Self::ParseInt(l0), Self::ParseInt(r0)) => l0 == r0,
            (Self::UrlParsing(l0), Self::UrlParsing(r0)) => l0 == r0,
            (Self::IOError(_), Self::IOError(_)) => true,
            (Self::TokioJoin(_), Self::TokioJoin(_)) => true,
            (Self::TomlDeserializeError(l0), Self::TomlDeserializeError(r0)) => l0 == r0,
            (Self::TomlSerializeError(l0), Self::TomlSerializeError(r0)) => l0 == r0,
            (Self::ReqwestError(_), Self::ReqwestError(_)) => true,
            (Self::RSSError(_), Self::RSSError(_)) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Error::IOError(other)
    }
}

impl From<toml::de::Error> for Error {
    fn from(other: toml::de::Error) -> Self {
        Error::TomlDeserializeError(other)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(other: toml::ser::Error) -> Self {
        Error::TomlSerializeError(other)
    }
}

impl From<ParseIntError> for Error {
    fn from(other: ParseIntError) -> Self {
        Error::ParseInt(other)
    }
}

impl From<url::ParseError> for Error {
    fn from(other: url::ParseError) -> Self {
        Error::UrlParsing(other)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(other: tokio::task::JoinError) -> Self {
        Error::TokioJoin(other)
    }
}

impl From<reqwest::Error> for Error {
    fn from(other: reqwest::Error) -> Self {
        Error::ReqwestError(other)
    }
}

impl From<rss::Error> for Error {
    fn from(other: rss::Error) -> Self {
        Error::RSSError(other)
    }
}
