use crate::Result;
use zenoh::subscriber::FlumeSubscriber;

pub struct Subscriber<'a> {
    pub subscriber: FlumeSubscriber<'a>,
}

impl<'a> Subscriber<'a> {
    pub async fn recv(&self) -> Result<String> {
        let sample = self.subscriber.recv_async().await?;
        Ok(format!("{}", sample.value))
    }
}
