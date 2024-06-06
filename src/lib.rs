use zenoh::prelude::r#async::*;

mod publisher;
mod subscriber;

pub use crate::publisher::Publisher;
pub use crate::subscriber::Subscriber;

/// This struct represents a node in the robotica system. This is the basic unit of interaction.
/// This is the basic unit of interaction with robotica. Use this to create channels (publishers,
/// subscribers, etc.), interact with the environment, and generally setup your application.
pub struct Node {
    _node_name: String,
    zenoh_session: Session,
}

impl Node {
    /// Creates a new node with a given name.
    ///
    /// # Errors
    /// This function will return an error if the zenoh session cannot be created.
    pub async fn new(node_name: String) -> Result<Node> {
        let zenoh_session = zenoh::open(config::default()).res().await?;
        Ok(Node {
            _node_name: node_name,
            zenoh_session,
        })
    }

    /// This function creates a subscriber for a given topic. The topic is a string that uniquely
    /// identifies the data channel across an entire system. Note that we expect the type to be a
    /// protobuf message that can be decoded.
    ///
    /// # Errors
    /// This function will return an error if the subscriber cannot be created. This usually means
    /// an error from zenoh.
    pub async fn subscribe<M: prost::Message + prost::Name + Default>(
        &self,
        topic: String,
    ) -> Result<Subscriber<'_, M>> {
        Subscriber::new_from_session(&self.zenoh_session, topic).await
    }

    /// This function creates a publisher for a given topic. The topic is a string that uniquely
    /// identifies the data channel across an entire system. Note that we expect the type to be a
    /// protobuf message that can be encoded.
    ///
    /// # Errors
    /// This function will return an error if the publisher cannot be created. This usually means
    /// an error from zenoh.
    pub async fn publish<M: prost::Message + prost::Name>(
        &self,
        topic: String,
    ) -> Result<Publisher<'_, M>> {
        Publisher::new_from_session(&self.zenoh_session, topic).await
    }
}

/// The full set of errors returned by this library. Please refer to the specific enum values for
/// the specific error types you should expect to get.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error propagated from zenoh
    #[error("zenoh error: {0}")]
    Zenoh(#[from] zenoh::Error),
    /// Error propagated from flume, usually at queue creation
    #[error("flume error: {0}")]
    Flume(#[from] flume::RecvError),
    /// This error is returned if, when receving a message from a subscriber, the type of the
    /// message does not match the type of the subscriber
    #[error("subscriber expected message of type \"{expected}\", but received message of type \"{actual}\"")]
    MismatchedSubscriberType { expected: String, actual: String },
    /// This error is returned if we fail to decode a protobuf when receiving a message on a
    /// subscriber. This usually means the data is invalid in some way, be it by corruption or for
    /// any other reason.
    #[error("error decoding protobuf: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),
}

/// A type alias for results returned by functions in this library.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;
