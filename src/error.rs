use reqwest;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    IO(reqwest::Error),
    Parse(serde_json::Error),
    Custom(Box<std::error::Error + Send + Sync>),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error {
            kind: ErrorKind::IO(err),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error {
            kind: ErrorKind::Parse(err),
        }
    }
}

impl<B> From<Box<B>> for Error
where
    B: std::error::Error + Send + Sync + 'static,
{
    fn from(err: Box<B>) -> Error {
        Error {
            kind: ErrorKind::Custom(err),
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "({:?})", self.kind)
    }
}

impl ::std::error::Error for Error {}
