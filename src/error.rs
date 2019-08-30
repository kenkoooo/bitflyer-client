use reqwest;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(ErrorKind);

#[derive(Debug)]
pub(crate) enum ErrorKind {
    IO(reqwest::Error),
    Parse(serde_json::Error, String),
    Custom(Box<std::error::Error + Send + Sync>),
}

impl Error {
    pub(crate) fn parse_error(err: serde_json::Error, content: &str) -> Self {
        Self(ErrorKind::Parse(err, content.to_string()))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error(ErrorKind::IO(err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error(ErrorKind::Parse(err, String::new()))
    }
}

impl<B> From<Box<B>> for Error
where
    B: std::error::Error + Send + Sync + 'static,
{
    fn from(err: Box<B>) -> Error {
        Error(ErrorKind::Custom(err))
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "({:?})", self.0)
    }
}

impl ::std::error::Error for Error {}
