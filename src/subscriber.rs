use crate::Result;
use std::io::Read;
use zenoh::prelude::r#async::*;

pub struct Subscriber<'a> {
    pub subscriber: zenoh::subscriber::FlumeSubscriber<'a>,
}

impl<'a> Subscriber<'a> {
    pub async fn recv(&self) -> Result<String> {
        let sample = self.subscriber.recv_async().await?;
        Ok(format!("{}", sample.value))
    }
}
