use crate::Result;
use prost::Message;
use prost_types::Timestamp;
use robotica_types::Header;
use std::{marker::PhantomData, time::SystemTime};
use zenoh::prelude::r#async::*;

/// This struct represents a publisher to a topic. This will require you send messages of type M.
/// Note that you cannot create this struct directly, but must instead fetch one from a
/// [`Node`](crate::Node).
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

    /// This function sends a message to the topic we're publishing to. Messages will be received
    /// by all subscribers to this topic.
    ///
    /// # Errors
    /// This function will return an error if the message cannot be sent for any reason. In
    /// practice, this means there was an error returned by zenoh when sending down the channel.
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
