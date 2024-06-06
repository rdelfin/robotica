use crate::{Error, Result};
use prost::Message;
use robotica_types::Header;
use std::marker::PhantomData;
use zenoh::{prelude::r#async::*, subscriber::FlumeSubscriber};

/// This struct represents a subscriber to a topic. This guarantees to return messages of type M.
/// Note that you cannot create this struct directly, but must instead fetch one from a
/// [`Node`](crate::Node).
pub struct Subscriber<'a, M: prost::Message + prost::Name + Default> {
    pub(crate) subscriber: FlumeSubscriber<'a>,
    pub(crate) _phantom: PhantomData<M>,
}

impl<'a, M: prost::Message + prost::Name + Default> Subscriber<'a, M> {
    pub(crate) async fn new_from_session(session: &'a Session, topic: String) -> Result<Self> {
        let subscriber = session.declare_subscriber(&topic).res().await?;
        Ok(Subscriber {
            subscriber,
            _phantom: PhantomData,
        })
    }

    /// This function blocks until a message is received on the topic we're subscribed to, per the
    /// `QoS` requirements of this subscriber.
    ///
    /// # Errors
    /// This function will return an error if the message cannot be received for any reason. In
    /// practice, this means either an error was returned by zenoh, or we failed to decode the
    /// protobuf data.
    pub async fn recv(&self) -> Result<ReceivedMessage<M>> {
        let sample = self.subscriber.recv_async().await?;
        let bytes = sample.value.payload.contiguous();
        let mut byte_ref = bytes.as_ref();
        let header = Header::decode_length_delimited(&mut byte_ref)?;
        if header.type_url == M::type_url() {
            Ok(ReceivedMessage {
                header,
                message: M::decode_length_delimited(&mut byte_ref)?,
            })
        } else {
            Err(Error::MismatchedSubscriberType {
                expected: M::type_url(),
                actual: header.type_url,
            })
        }
    }
}

pub struct ReceivedMessage<M> {
    pub header: Header,
    pub message: M,
}
