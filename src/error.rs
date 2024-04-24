use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // index errors
    #[error("Bad Index Record")]
    BadIndexRecord,

    // url errors
    #[error("Missing host: {0}")]
    MissingHost(String),

    // Page loading errors
    #[error("Page not found (404): {0}")]
    PageNotFound(String),

    // Ingester errors
    #[error("Unknown ingester {0}")]
    UnknownIngester(String),
    #[error("Missing base url")]
    MissingBaseUrl,
    #[error("Could not find a date for the last wikipedia dump")]
    WikipediaMissingDate,
    #[error("Parser got into an invalid state: {0}")]
    InvalidState(String),
    #[error("Http request error ({0})")]
    Request(reqwest::StatusCode),

    // File IO Errors
    #[error("Incomplete write. This is bad, but potentially fixable. File: {0:?} byte index:{1}")]
    IncompleteWrite(String, u64),

    // Std lib errors
    #[error("Could not parse int: {0:?}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Could not parse url: {0:?}")]
    UrlParsing(#[from] url::ParseError),
    #[error("IO Error: {0:?}")]
    IOError(#[from] std::io::Error),
    #[error("Invalid utf8 characters: {0:?}")]
    EncodingError(#[from] std::string::FromUtf8Error),

    // dependencies errors
    #[error("Toml Deserialize Error: {0:?}")]
    TomlDeserializeError(#[from] toml::de::Error),
    #[error("Toml Serialize Error: {0:?}")]
    TomlSerializeError(#[from] toml::ser::Error),
    #[error("Could not parse time: {0:?}")]
    TimeParsing(#[from] time::error::Parse),
    #[error("Could not format time: {0:?}")]
    TimeFormatting(#[from] time::error::Format),
    #[error("Could not do something with sled: {0:?}")]
    SledError(#[from] sled::Error),
    #[error("Reqwest error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("RSS error: {0:?}")]
    RSSError(#[from] rss::Error),
    #[error("Could not join: {0:?}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("Could not parse xml: {0:?}")]
    InvalidXML(#[from] quick_xml::Error),
    #[error("Template error: {0:?}")]
    Tera(#[from] tera::Error),

    // Log4rs errors
    #[error("Could not set up logger: {0:?}")]
    LogSetup(#[from] log::SetLoggerError),
    #[error("Could not set up logger config: {0:?}")]
    LogConfig(#[from] log4rs::config::runtime::ConfigErrors),
    #[error("Could not set up logger config, bad log level: {0:?}")]
    LogLevel(#[from] log::ParseLevelError),
    #[error("anyhow: {0:?}")]
    Anyhow(#[from] anyhow::Error),
}

impl PartialEq for Error {
    /// Note: This should only be used in very limited cases
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PageNotFound(l0), Self::PageNotFound(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let message = self.to_string();
        actix_web::HttpResponse::build(self.status_code()).body(message)
    }
}
