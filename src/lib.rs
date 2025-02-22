use log::LevelFilter;
use simple_logger::SimpleLogger;
use tracing::info;
use zenoh::Session;

pub use log;
pub use tracing;

mod proto;
mod publisher;
mod subscriber;

pub use crate::publisher::{Publisher, UntypedPublisher};
pub use crate::subscriber::{Subscriber, UntypedSubscriber};

/// This struct represents a node in the robotica system. This is the basic unit of interaction.
/// This is the basic unit of interaction with robotica. Use this to create channels (publishers,
/// subscribers, etc.), interact with the environment, and generally setup your application.
pub struct Node {
    node_name: String,
    zenoh_session: Session,
    file_descriptor: Vec<Vec<u8>>,
}

impl Node {
    /// Creates a new node with logging enabled and a given name.
    ///
    /// # Errors
    /// This function will return an error if the zenoh session cannot be created, or if logging
    /// setup fails.
    pub async fn new_with_logging<S: AsRef<str>>(node_name: S, logging: LogConfig) -> Result<Node> {
        configure_logging(&logging)?;
        Self::new(node_name).await
    }

    /// Creates a new node with a given name.
    ///
    /// # Errors
    /// This function will return an error if the zenoh session cannot be created.
    pub async fn new<S: AsRef<str>>(node_name: S) -> Result<Node> {
        let zenoh_session = zenoh::open(zenoh::Config::default()).await?;
        info!(msg = "node_created", name = node_name.as_ref());
        Ok(Node {
            node_name: node_name.as_ref().into(),
            zenoh_session,
            // We default to use our own file descriptor
            file_descriptor: vec![robotica_types::DESCRIPTOR_SET_BYTES.to_vec()],
        })
    }

    /// This function allows you to override the file descriptor data used for untyped publishers
    /// and subscribers, as well as other relevant reflection functions.
    pub fn add_file_descriptors(&mut self, file_descriptors_bytes: &[u8]) {
        self.file_descriptor.push(file_descriptors_bytes.to_vec());
    }

    /// This function creates a subscriber for a given topic. The topic is a string that uniquely
    /// identifies the data channel across an entire system. Note that we expect the type to be a
    /// protobuf message that can be decoded.
    ///
    /// # Errors
    /// This function will return an error if the subscriber cannot be created. This usually means
    /// an error from zenoh.
    pub async fn subscribe<M: prost::Message + prost::Name + Default, S: AsRef<str>>(
        &self,
        topic: S,
    ) -> Result<Subscriber<M>> {
        let topic = topic.as_ref();
        let sub = Subscriber::new_from_session(&self.zenoh_session, topic).await?;
        info!(
            msg = "subscriber_created",
            name = self.node_name,
            topic = topic,
            type_url = M::type_url(),
        );
        Ok(sub)
    }

    /// This function creates an untyped subscriber for a given topic. The topic is a string that
    /// uniquely identifies the data channel across an entire system. The subscriber will attempt
    /// to dynamically decode the messages it receives by searching for a protobuf that matches the
    /// type URL of the message in the provided file descriptors.
    ///
    /// # Errors
    /// This function will return an error if the subscriber cannot be created. This usually means
    /// an error from zenoh.
    pub async fn subscribe_untyped<S: AsRef<str>>(&self, topic: S) -> Result<UntypedSubscriber> {
        let topic = topic.as_ref();
        let sub =
            UntypedSubscriber::new_from_session(&self.zenoh_session, topic, &self.file_descriptor)
                .await?;
        info!(
            msg = "subscriber_created",
            name = self.node_name,
            topic = topic,
            type_url = "unknown",
        );
        Ok(sub)
    }

    /// This function creates a publisher for a given topic. The topic is a string that uniquely
    /// identifies the data channel across an entire system. Note that we expect the type to be a
    /// protobuf message that can be encoded.
    ///
    /// # Errors
    /// This function will return an error if the publisher cannot be created. This usually means
    /// an error from zenoh.
    pub async fn publish<M: prost::Message + prost::Name, S: AsRef<str>>(
        &self,
        topic: S,
    ) -> Result<Publisher<'_, M>> {
        let topic = topic.as_ref();
        let publisher = Publisher::new_from_session(&self.zenoh_session, topic).await?;
        info!(
            msg = "publisher_created",
            name = self.node_name,
            topic = topic,
            type_url = M::type_url(),
        );
        Ok(publisher)
    }

    /// This function creates a dynamically-typed publisher for a given topic. The topic is a
    /// string that uniquely identifies the data channel across an entire system. Note that we
    /// expect the type to be specified ahead of time in the `type_url` parameter, and any
    /// published messages should have matching JSON data, as per the official [JSON
    /// mapping](https://protobuf.dev/programming-guides/proto3/#json).
    ///
    /// # Errors
    /// This function will return an error if the publisher cannot be created. This usually means
    /// an error from zenoh, or that the type URL doesn't exist in the provided file descriptors.
    pub async fn publish_untyped<S: AsRef<str>, S2: AsRef<str>>(
        &self,
        topic: S,
        type_url: S2,
    ) -> Result<UntypedPublisher<'_>> {
        let topic = topic.as_ref();
        let type_url = type_url.as_ref();
        let publisher = UntypedPublisher::new_from_session(
            &self.zenoh_session,
            topic,
            type_url,
            &self.file_descriptor,
        )
        .await?;
        info!(
            msg = "publisher_created",
            name = self.node_name,
            topic = topic,
            type_url = type_url,
        );
        Ok(publisher)
    }
}

/// Configuration for the logging setup
pub struct LogConfig {
    default_level: LevelFilter,
    zenoh_level: LevelFilter,
    robotica_level: LevelFilter,
}

impl LogConfig {
    /// Create a log config with default values
    #[must_use]
    pub fn new() -> LogConfig {
        Self::default()
    }

    /// Sets the level to log at as default.
    #[must_use]
    pub fn default_level(mut self, l: LevelFilter) -> LogConfig {
        self.default_level = l;
        self
    }

    /// Sets the level to log the zenoh internals at.
    #[must_use]
    pub fn zenoh_level(mut self, l: LevelFilter) -> LogConfig {
        self.zenoh_level = l;
        self
    }

    /// Sets the level to log the robotica internals at.
    #[must_use]
    pub fn robotica_level(mut self, l: LevelFilter) -> LogConfig {
        self.robotica_level = l;
        self
    }
}

impl Default for LogConfig {
    fn default() -> LogConfig {
        LogConfig {
            default_level: LevelFilter::Info,
            zenoh_level: LevelFilter::Warn,
            robotica_level: LevelFilter::Info,
        }
    }
}

fn configure_logging(log_config: &LogConfig) -> Result<()> {
    SimpleLogger::new()
        .with_level(log_config.default_level)
        .with_module_level("zenoh", log_config.zenoh_level)
        .with_module_level("robotica", log_config.robotica_level)
        .init()?;
    Ok(())
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
    /// Failure when reading a protobuf descriptor. This error is thrown during untyped subscriber
    /// creation.
    #[error("error reading protobuf descriptor: {0}")]
    ProtobufDescriptorRead(#[from] prost_reflect::DescriptorError),
    /// A message was received with an invalid type URL, either because the type does not exist or
    /// because we cannot parse it.
    #[error("invalid type URL: {0}")]
    InvalidTypeUrl(String),
    /// Error when parsing the JSON provided in the dynamic publisher.
    #[error("invalid type URL: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    /// Error when parsing the JSON provided in the dynamic publisher.
    #[error("error with logging: {0}")]
    LogSetupError(#[from] log::SetLoggerError),
}

/// A type alias for results returned by functions in this library.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;
