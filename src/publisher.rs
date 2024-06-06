use crate::Result;
use prost::Message;
use prost_types::Timestamp;
use robotica_types::Header;
use std::{marker::PhantomData, time::SystemTime};
use zenoh::prelude::r#async::*;

pub struct Publisher<'a, M: prost::Message + prost::Name> {
    pub publisher: zenoh::publication::Publisher<'a>,
    pub _phantom: PhantomData<M>,
}

impl<'a, M: prost::Message + prost::Name> Publisher<'a, M> {
    pub(crate) async fn new_from_session(session: &'a Session, topic: String) -> Result<Self> {
        let publisher = session.declare_publisher(topic).res().await?;
        Ok(Publisher {
            publisher,
            _phantom: PhantomData,
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub async fn send(&self, message: &M) -> Result<()> {
        let header = Header {
            message_timestamp: Some(Timestamp::from(SystemTime::now())),
            type_url: M::type_url(),
        };
        let mut buf = header.encode_length_delimited_to_vec();
        buf.extend_from_slice(&message.encode_length_delimited_to_vec());
        self.publisher.put(buf).res().await?;
        Ok(())
    }
}
