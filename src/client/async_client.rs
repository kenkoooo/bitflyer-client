use crate::{AsyncResponse, Result};
use serde::Deserialize;
use ws::{self, CloseCode, Error, ErrorKind, Handler, Handshake, Message, Sender};

const JSON_RPC_URL: &str = "wss://ws.lightstream.bitflyer.com/json-rpc";

pub const CHANNEL_BOARD: &str = "lightning_board_BTC_JPY";
pub const CHANNEL_EXECUTIONS: &str = "lightning_executions_BTC_JPY";

pub trait AsyncResponseHandler {
    fn receive_response(&mut self, response: AsyncResponse) -> Result<()>;
    fn get_channel(&self) -> &str;
    fn is_closed(&self) -> bool;
}

pub fn async_connect<F, I>(factory: F) -> Result<()>
where
    F: Fn() -> I,
    I: AsyncResponseHandler,
{
    ws::connect(JSON_RPC_URL, |out| WrapperClient {
        inner_handler: factory(),
        out,
    })
    .map_err(|e| Box::new(e))?;
    Ok(())
}

struct WrapperClient<I> {
    inner_handler: I,
    out: Sender,
}

impl<I> Handler for WrapperClient<I>
where
    I: AsyncResponseHandler,
{
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        let request = format!(
            r#"{{"method":"subscribe", "params":{{"channel":"{}"}} }}"#,
            self.inner_handler.get_channel()
        );
        self.out.send(request)
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        match msg {
            Message::Text(content) => {
                let response = serde_json::from_str::<JsonRpcResponse>(&content)
                    .map_err(|e| Error::new(ErrorKind::Custom(Box::new(e)), ""))?;
                self.inner_handler
                    .receive_response(response.params.message)
                    .map_err(|e| Error::new(ErrorKind::Custom(Box::new(e)), ""))?
            }
            _ => {}
        }
        if self.inner_handler.is_closed() {
            self.out.close(CloseCode::Normal)
        } else {
            Ok(())
        }
    }
}

#[derive(Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    method: String,
    params: JsonRpcResponseParams,
}

#[derive(Deserialize, Debug)]
struct JsonRpcResponseParams {
    channel: String,
    message: AsyncResponse,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_async_connect_board() {
        struct Client {
            count: AtomicUsize,
        }

        impl AsyncResponseHandler for Client {
            fn receive_response(&mut self, response: AsyncResponse) -> Result<()> {
                eprintln!("{:?}", response);
                self.count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }

            fn get_channel(&self) -> &str {
                CHANNEL_BOARD
            }

            fn is_closed(&self) -> bool {
                self.count.load(Ordering::Relaxed) == 10
            }
        }

        async_connect(|| Client {
            count: AtomicUsize::new(0),
        })
        .unwrap();
    }

    #[test]
    fn test_async_connect_executions() {
        struct Client {
            count: AtomicUsize,
        }

        impl AsyncResponseHandler for Client {
            fn receive_response(&mut self, response: AsyncResponse) -> Result<()> {
                eprintln!("{:?}", response);
                self.count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }

            fn get_channel(&self) -> &str {
                CHANNEL_EXECUTIONS
            }

            fn is_closed(&self) -> bool {
                self.count.load(Ordering::Relaxed) == 3
            }
        }

        async_connect(|| Client {
            count: AtomicUsize::new(0),
        })
        .unwrap();
    }
}
