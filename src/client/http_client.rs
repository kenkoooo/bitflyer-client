use crate::error::Error;
use crate::Result;
use crate::{Board, ExchangeHistory};
use reqwest;
use reqwest::header::ACCEPT_ENCODING;
use serde::de::DeserializeOwned;

const BASE_URL: &str = "https://api.bitflyer.com";

pub struct HttpBitFlyerClient {
    client: reqwest::Client,
}

impl Default for HttpBitFlyerClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl HttpBitFlyerClient {
    fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let content = self
            .client
            .get(url)
            .header(ACCEPT_ENCODING, "gzip")
            .send()?
            .text()?;
        let result = serde_json::from_str(&content).map_err(|e| Error::parse_error(e, &content))?;
        Ok(result)
    }

    pub fn fetch_board(&self) -> Result<Board> {
        let url = format!("{base_url}/v1/board", base_url = BASE_URL);
        self.get(url.as_str())
    }

    pub fn fetch_history(&self, before: Option<i64>) -> Result<Vec<ExchangeHistory>> {
        let before = before.unwrap_or(0);
        let url = format!(
            "{base_url}/v1/executions?count=500&before={before}",
            base_url = BASE_URL,
            before = before,
        );
        self.get(url.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_board() {
        let client = HttpBitFlyerClient::default();
        assert!(client.fetch_board().is_ok());
    }

    #[test]
    fn test_fetch_history() {
        let client = HttpBitFlyerClient::default();
        assert!(client.fetch_history(None).is_ok());
    }
}
