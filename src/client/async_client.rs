use crate::Result;
use ws::{self, CloseCode, Error, ErrorKind, Handler, Handshake, Message, Sender};

const JSON_RPC_URL: &str = "wss://ws.lightstream.bitflyer.com/json-rpc";

pub trait ResponseHandler {
    fn on_text_message(&mut self, text: String) -> Result<()>;
    fn get_channel(&self) -> &str;
    fn is_closed(&self) -> bool;
}

struct WrapperClient<I> {
    inner_handler: I,
    out: Sender,
}

impl<I> Handler for WrapperClient<I>
where
    I: ResponseHandler,
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
            Message::Text(content) => self
                .inner_handler
                .on_text_message(content)
                .map_err(|e| Error::new(ErrorKind::Custom(Box::new(e)), ""))?,
            _ => {}
        }
        if self.inner_handler.is_closed() {
            self.out.close(CloseCode::Normal)
        } else {
            Ok(())
        }
    }
}

fn connect<F, I>(factory: F) -> Result<()>
where
    F: Fn() -> I,
    I: ResponseHandler,
{
    ws::connect(JSON_RPC_URL, |out| WrapperClient {
        inner_handler: factory(),
        out,
    })
    .map_err(|e| Box::new(e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Client {
        count: AtomicUsize,
    }

    impl ResponseHandler for Client {
        fn on_text_message(&mut self, text: String) -> Result<()> {
            eprintln!("{}", text);
            self.count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        fn get_channel(&self) -> &str {
            "lightning_board_BTC_JPY"
        }

        fn is_closed(&self) -> bool {
            self.count.load(Ordering::Relaxed) == 10
        }
    }

    #[test]
    fn test_connect() {
        connect(|| Client {
            count: AtomicUsize::new(0),
        })
        .unwrap();
    }
}
