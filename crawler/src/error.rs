use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Ceridwen(ceridwen::error::Error),

    #[error("Unknown ingester {0}")]
    UnknownIngester(String),
    #[error("Missing base url")]
    MissingBaseUrl,
    #[error("Could not find a date for the last wikipedia dump")]
    WikipediaMissingDate,

    #[error("Parser got into an invalid state: {0}")]
    InvalidState(String),

    // url errors
    #[error("Missing host: {0}")]
    MissingHost(String),

    // Page loading errors
    #[error("Page not found (404): {0}")]
    PageNotFound(String),

    #[error("Http request error ({0})")]
    Request(StatusCode),

    #[error("Incomplete write! {0}:{1}")]
    IncompleteWrite(String, u64),

    #[error("encoding error: {0}")]
    Utf8Error(std::string::FromUtf8Error),

    #[error("Could not parse int: {0:?}")]
    ParseInt(std::num::ParseIntError),
    #[error("IO Error: {0:?}")]
    IOError(std::io::Error),
    #[error("Could not parse url: {0:?}")]
    UrlParsing(url::ParseError),
    #[error("Reqwest error: {0:?}")]
    ReqwestError(reqwest::Error),
    #[error("RSS error: {0:?}")]
    RSSError(rss::Error),
    #[error("Could not parse time: {0:?}")]
    TimeParsing(time::error::Parse),
    #[error("Could not format time: {0:?}")]
    TimeFormatting(time::error::Format),
    #[error("Could not join: {0:?}")]
    TokioJoin(tokio::task::JoinError),
    #[error("Could not parse xml: {0:?}")]
    InvalidXML(quick_xml::Error),

    // Log4rs errors
    #[error("Could not set up logger: {0:?}")]
    LogSetup(log::SetLoggerError),
    #[error("Could not set up logger config: {0:?}")]
    LogConfig(log4rs::config::runtime::ConfigErrors),
    #[error("anyhow: {0:?}")]
    Anyhow(anyhow::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ceridwen(a), Self::Ceridwen(b)) => a == b,
            (Self::UnknownIngester(l0), Self::UnknownIngester(r0)) => l0 == r0,
            (Self::MissingBaseUrl, Self::MissingBaseUrl) => true,
            (Self::WikipediaMissingDate, Self::WikipediaMissingDate) => true,
            (Self::MissingHost(a), Self::MissingHost(b)) => a == b,
            (Self::PageNotFound(a), Self::PageNotFound(b)) => a == b,
            (Self::ParseInt(a), Self::ParseInt(b)) => a == b,
            (Self::UrlParsing(a), Self::UrlParsing(b)) => a == b,
            (Self::IOError(_), Self::IOError(_)) => true,
            (Self::TimeParsing(_), Self::TimeParsing(_)) => true,
            (Self::TokioJoin(_), Self::TokioJoin(_)) => true,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl From<ceridwen::error::Error> for Error {
    fn from(value: ceridwen::error::Error) -> Self {
        Error::Ceridwen(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(other: std::io::Error) -> Self {
        Error::IOError(other)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(other: std::num::ParseIntError) -> Self {
        Error::ParseInt(other)
    }
}

impl From<url::ParseError> for Error {
    fn from(other: url::ParseError) -> Self {
        Error::UrlParsing(other)
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

impl From<time::error::Parse> for Error {
    fn from(other: time::error::Parse) -> Self {
        Error::TimeParsing(other)
    }
}

impl From<time::error::Format> for Error {
    fn from(other: time::error::Format) -> Self {
        Error::TimeFormatting(other)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(other: tokio::task::JoinError) -> Self {
        Error::TokioJoin(other)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(other: std::string::FromUtf8Error) -> Self {
        Error::Utf8Error(other)
    }
}

impl From<quick_xml::Error> for Error {
    fn from(other: quick_xml::Error) -> Self {
        Error::InvalidXML(other)
    }
}

impl From<log::SetLoggerError> for Error {
    fn from(other: log::SetLoggerError) -> Self {
        Error::LogSetup(other)
    }
}

impl From<log4rs::config::runtime::ConfigErrors> for Error {
    fn from(other: log4rs::config::runtime::ConfigErrors) -> Self {
        Error::LogConfig(other)
    }
}

impl From<anyhow::Error> for Error {
    fn from(other: anyhow::Error) -> Self {
        Error::Anyhow(other)
    }
}
