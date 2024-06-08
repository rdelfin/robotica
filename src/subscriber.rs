use crate::{Error, Result};
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, MessageDescriptor};
use robotica_types::Header;
use std::marker::PhantomData;
use zenoh::{prelude::r#async::*, subscriber::FlumeSubscriber};

/// This struct represents a subscriber to a topic. This guarantees to return messages of type M.
/// Note that you cannot create this struct directly, but must instead fetch one from a
/// [`Node`](crate::Node).
pub struct Subscriber<'a, M: prost::Message + prost::Name + Default> {
    subscriber: FlumeSubscriber<'a>,
    _phantom: PhantomData<M>,
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

#[allow(clippy::module_name_repetitions)]
pub struct UntypedSubscriber<'a> {
    subscriber: FlumeSubscriber<'a>,
    file_descriptor_pools: Vec<DescriptorPool>,
    active_message_descriptor: Option<(String, MessageDescriptor)>,
}

impl<'a> UntypedSubscriber<'a> {
    pub(crate) async fn new_from_session<I: IntoIterator<Item = &'static [u8]>>(
        session: &'a Session,
        topic: String,
        file_descriptors_bytes: I,
    ) -> Result<Self> {
        let subscriber = session.declare_subscriber(&topic).res().await?;
        let file_descriptor_pools = file_descriptors_bytes
            .into_iter()
            .map(DescriptorPool::decode)
            .collect::<Result<Vec<_>, _>>()?;
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
        let message_name = type_url
            .split('/')
            .nth(1)
            .ok_or_else(|| Error::InvalidTypeUrl(type_url.into()))?;

        self.active_message_descriptor = if let Some((active_message_name, message_descriptor)) =
            self.active_message_descriptor.take()
        {
            if active_message_name == message_name {
                Some((active_message_name, message_descriptor))
            } else {
                Some((type_url.into(), self.search_file_descriptors(message_name)?))
            }
        } else {
            Some((type_url.into(), self.search_file_descriptors(message_name)?))
        };

        Ok(&self
            .active_message_descriptor
            .as_ref()
            .expect("active message descriptor must be set here")
            .1)
    }

    fn search_file_descriptors(&self, message_name: &str) -> Result<MessageDescriptor> {
        self.file_descriptor_pools
            .iter()
            .find_map(|pool| pool.get_message_by_name(message_name))
            .ok_or_else(|| Error::InvalidTypeUrl(message_name.into()))
    }
}

pub struct ReceivedMessage<M> {
    pub header: Header,
    pub message: M,
}
