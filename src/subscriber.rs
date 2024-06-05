use crate::Result;
use zenoh::prelude::r#async::*;

pub struct Subscriber<'a> {
    pub subscriber: zenoh::subscriber::FlumeSubscriber<'a>,
}

impl<'a> Subscriber<'a> {
    pub async fn recv(&self) -> Result<String> {
        let sample = self.subscriber.recv_async().await?;
        let byte_reader = sample.value.payload.reader();
        Ok(std::io::read_to_string(byte_reader).expect("failed to read sample"))
    }
}
