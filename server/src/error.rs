use actix_web::HttpResponse;
use actix_web::ResponseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Ceridwen(ceridwen::error::Error),

    #[error("Could not parse int: {0:?}")]
    ParseInt(std::num::ParseIntError),
    #[error("IO Error: {0:?}")]
    IOError(std::io::Error),
    #[error("Template error: {0:?}")]
    Tera(tera::Error),
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

impl From<tera::Error> for Error {
    fn from(other: tera::Error) -> Self {
        Error::Tera(other)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let message = self.to_string();
        HttpResponse::build(self.status_code()).body(message)
    }
}
