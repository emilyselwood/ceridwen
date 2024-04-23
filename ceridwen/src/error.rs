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
    #[error("Could not open datafile: {0:?} because: {1:?}")]
    BadFileName(String, std::io::Error),

    // url errors
    #[error("Missing host: {0}")]
    MissingHost(String),

    // Page loading errors
    #[error("Page not found (404): {0}")]
    PageNotFound(String),

    // File IO Errors
    #[error("Incomplete write. This is bad, but potentially fixable. File: {0:?} byte index:{1}")]
    IncompleteWrite(String, u64),

    // Std lib errors
    #[error("Could not parse int: {0:?}")]
    ParseInt(std::num::ParseIntError),
    #[error("Could not parse url: {0:?}")]
    UrlParsing(url::ParseError),
    #[error("IO Error: {0:?}")]
    IOError(std::io::Error),
    #[error("Invalid utf8 characters: {0:?}")]
    EncodingError(std::string::FromUtf8Error),

    // dependencies errors
    #[error("Toml Deserialize Error: {0:?}")]
    TomlDeserializeError(toml::de::Error),
    #[error("Toml Serialize Error: {0:?}")]
    TomlSerializeError(toml::ser::Error),
    #[error("Could not parse time: {0:?}")]
    TimeParsing(time::error::Parse),
    #[error("Could not format time: {0:?}")]
    TimeFormatting(time::error::Error),
    #[error("Could not do something with sled: {0:?}")]
    SledError(sled::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::IndexDirAlreadyExists(l0), Self::IndexDirAlreadyExists(r0)) => l0 == r0,
            (Self::IndexDirDoesNotExist(l0), Self::IndexDirDoesNotExist(r0)) => l0 == r0,
            (Self::MissingHost(l0), Self::MissingHost(r0)) => l0 == r0,
            (Self::PageNotFound(l0), Self::PageNotFound(r0)) => l0 == r0,
            (Self::ParseInt(l0), Self::ParseInt(r0)) => l0 == r0,
            (Self::UrlParsing(l0), Self::UrlParsing(r0)) => l0 == r0,
            (Self::IOError(_), Self::IOError(_)) => true,
            (Self::TomlDeserializeError(l0), Self::TomlDeserializeError(r0)) => l0 == r0,
            (Self::TomlSerializeError(l0), Self::TomlSerializeError(r0)) => l0 == r0,
            (Self::TimeParsing(_), Self::TimeParsing(_)) => true,
            (Self::TimeFormatting(_), Self::TimeFormatting(_)) => true,

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

impl From<time::error::Parse> for Error {
    fn from(other: time::error::Parse) -> Self {
        Error::TimeParsing(other)
    }
}

impl From<time::error::Error> for Error {
    fn from(other: time::error::Error) -> Self {
        Error::TimeFormatting(other)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(other: std::string::FromUtf8Error) -> Self {
        Error::EncodingError(other)
    }
}

impl From<sled::Error> for Error {
    fn from(other: sled::Error) -> Self {
        Error::SledError(other)
    }
}
