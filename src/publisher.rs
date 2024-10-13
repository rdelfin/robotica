use crate::{
    proto::{parse_file_descriptors, search_file_descriptors},
    Result,
};
use prost::Message;
use prost_reflect::{DynamicMessage, MessageDescriptor};
use prost_types::Timestamp;
use robotica_types::Header;
use serde_json::Value;
use std::{marker::PhantomData, time::SystemTime};
use zenoh::prelude::r#async::*;

/// This struct represents a publisher to a topic. This will require you send messages of type M.
/// Note that you cannot create this struct directly, but must instead fetch one from a
/// [`Node`](crate::Node).
pub struct Publisher<'a, M: prost::Message + prost::Name> {
    publisher: zenoh::publication::Publisher<'a>,
    _phantom: PhantomData<M>,
}

impl<'a, M: prost::Message + prost::Name> Publisher<'a, M> {
    pub(crate) async fn new_from_session<S: AsRef<str>>(
        session: &'a Session,
        topic: S,
    ) -> Result<Self> {
        let publisher = session
            .declare_publisher(topic.as_ref().to_string())
            .res()
            .await?;
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

/// This struct represents a dynamically-typed publisher to a topic. This expects the JSON value
/// provided at publish time to be deserializeable into the correct protobuf message. Note that you
/// cannot create this struct directly, but must instead fetch one from a [`Node`](crate::Node).
#[allow(clippy::module_name_repetitions)]
pub struct UntypedPublisher<'a> {
    publisher: zenoh::publication::Publisher<'a>,
    message_descriptor: MessageDescriptor,
    type_url: String,
}

impl<'a> UntypedPublisher<'a> {
    pub(crate) async fn new_from_session<S: AsRef<str>, S2: AsRef<str>>(
        session: &'a Session,
        topic: S,
        type_url: S2,
        file_descriptors_bytes: &[&[u8]],
    ) -> Result<UntypedPublisher<'a>> {
        let type_url = type_url.as_ref();
        let file_descriptor_pools = parse_file_descriptors(file_descriptors_bytes)?;
        let message_descriptor = search_file_descriptors(&file_descriptor_pools, type_url)?;
        let publisher = session
            .declare_publisher(topic.as_ref().to_string())
            .res()
            .await?;
        Ok(UntypedPublisher {
            publisher,
            message_descriptor,
            type_url: type_url.into(),
        })
    }

    /// This function sends a message to the topic we're publishing to. Messages will be received
    /// by all subscribers to this topic. Note we expect a dynamic message as input that will be
    /// parsed and encoded based on the type URL provided at creation time.
    ///
    /// # Errors
    /// This function will return an error if the message cannot be sent for any reason. In
    /// practice, this means there was an error returned by zenoh when sending down the channel, or
    /// an error while attempting to encode the message dynamically.
    pub async fn send(&self, json_value: Value) -> Result<()> {
        let json_string = json_value.to_string();
        let mut deserializer = serde_json::Deserializer::from_str(&json_string);
        let dyn_message =
            DynamicMessage::deserialize(self.message_descriptor.clone(), &mut deserializer)?;

        let header = Header {
            message_timestamp: Some(Timestamp::from(SystemTime::now())),
            type_url: self.type_url.clone(),
        };
        let mut buf = header.encode_length_delimited_to_vec();
        buf.extend_from_slice(&dyn_message.encode_length_delimited_to_vec());
        self.publisher.put(buf).res().await?;
        Ok(())
    }
}
