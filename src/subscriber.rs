use crate::{
    proto::{parse_file_descriptors, search_file_descriptors},
    Error, Result,
};
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, MessageDescriptor};
use robotica_types::Header;
use std::marker::PhantomData;
use tracing::instrument;
use zenoh::{prelude::r#async::*, subscriber::FlumeSubscriber};

/// This struct represents a subscriber to a topic. This guarantees to return messages of type M.
/// Note that you cannot create this struct directly, but must instead fetch one from a
/// [`Node`](crate::Node).
pub struct Subscriber<'a, M: prost::Message + prost::Name + Default> {
    subscriber: FlumeSubscriber<'a>,
    _phantom: PhantomData<M>,
}

impl<'a, M: prost::Message + prost::Name + Default> Subscriber<'a, M> {
    pub(crate) async fn new_from_session<S: AsRef<str>>(
        session: &'a Session,
        topic: S,
    ) -> Result<Self> {
        let subscriber = session.declare_subscriber(topic.as_ref()).res().await?;
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
    #[instrument(level = "trace", skip_all)]
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

#[allow(clippy::module_name_repetitions)]
pub struct UntypedSubscriber<'a> {
    subscriber: FlumeSubscriber<'a>,
    file_descriptor_pools: Vec<DescriptorPool>,
    active_message_descriptor: Option<(String, MessageDescriptor)>,
}

impl<'a> UntypedSubscriber<'a> {
    pub(crate) async fn new_from_session<S: AsRef<str>>(
        session: &'a Session,
        topic: S,
        file_descriptors_bytes: &[Vec<u8>],
    ) -> Result<Self> {
        let subscriber = session.declare_subscriber(topic.as_ref()).res().await?;
        let file_descriptor_pools = parse_file_descriptors(file_descriptors_bytes)?;
        Ok(UntypedSubscriber {
            subscriber,
            file_descriptor_pools,
            active_message_descriptor: None,
        })
    }

    /// This function blocks until a message is received on the topic we're subscribed to, per the
    /// `QoS` requirements of this subscriber. Note the return type is a
    /// [`prost_reflect::DynamicMessage`], which can be queried for type information or serialized
    /// using `serde`.
    ///
    /// # Errors
    /// This function will return an error if the message cannot be received for any reason. In
    /// practice, this means either an error was returned by zenoh, or we failed to decode the
    /// protobuf data. Note that because this is an untyped subscriber, we do a best-effort attempt
    /// at matching the type, but if two messages in the file descriptors have the same exact name,
    /// we could end up decoding the wrong message silently.
    ///
    /// # Panics
    /// This function will only panic if a u64 cannot be converted to a usize on your system.
    #[instrument(level = "trace", skip_all)]
    pub async fn recv(&mut self) -> Result<ReceivedMessage<DynamicMessage>> {
        // Fetch message bytes and decode the header
        let sample = self.subscriber.recv_async().await?;
        let bytes = sample.value.payload.contiguous();
        let mut byte_ref = bytes.as_ref();
        let header = Header::decode_length_delimited(&mut byte_ref)?;

        // Fetch the appropriate message descriptor
        let message_descriptor = self.get_message_descriptor(&header.type_url)?;

        // Since messages are length-delimited, we need to read a single varint first
        let len = usize::try_from(prost::encoding::decode_varint(&mut byte_ref)?)
            .expect("u64 should always fit in usize");

        Ok(ReceivedMessage {
            header,
            message: DynamicMessage::decode(message_descriptor.clone(), &byte_ref[..len])?,
        })
    }

    fn get_message_descriptor(&mut self, type_url: &str) -> Result<&MessageDescriptor> {
        self.active_message_descriptor = if let Some((active_type_url, message_descriptor)) =
            self.active_message_descriptor.take()
        {
            if active_type_url == type_url {
                Some((active_type_url, message_descriptor))
            } else {
                Some((
                    type_url.into(),
                    search_file_descriptors(&self.file_descriptor_pools, type_url)?,
                ))
            }
        } else {
            Some((
                type_url.into(),
                search_file_descriptors(&self.file_descriptor_pools, type_url)?,
            ))
        };

        Ok(&self
            .active_message_descriptor
            .as_ref()
            .expect("active message descriptor must be set here")
            .1)
    }
}

pub struct ReceivedMessage<M> {
    pub header: Header,
    pub message: M,
}
