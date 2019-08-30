use bitflyer_client::{
    async_connect, AsyncResponse, AsyncResponseHandler, Result, CHANNEL_BOARD, CHANNEL_EXECUTIONS,
};

struct Client {}
impl AsyncResponseHandler for Client {
    fn receive_response(&mut self, response: AsyncResponse) -> Result<()> {
        eprintln!("{:?}", response);
        Ok(())
    }

    fn get_channel(&self) -> &str {
        CHANNEL_BOARD
    }

    fn is_closed(&self) -> bool {
        false
    }
}

fn main() {
    async_connect(|| Client {});
}
