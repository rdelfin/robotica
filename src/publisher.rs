use crate::Result;
use zenoh::prelude::r#async::*;

pub struct Publisher<'a> {
    pub publisher: zenoh::publication::Publisher<'a>,
}

impl<'a> Publisher<'a> {
    pub async fn send(&self, value: String) -> Result<()> {
        self.publisher.put(value).res().await?;
        Ok(())
    }
}
