use log::LevelFilter;
use prost::Message;
use robotica_types::{PublisherInfo, PublisherList, SubscriberInfo, SubscriberList};
use simple_logger::SimpleLogger;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use zenoh::query::ConsolidationMode;
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
    pubsub_data: Arc<Mutex<PubsubData>>,
    file_descriptor: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct PubsubData {
    publishers: HashSet<String>,
    subscribers: HashSet<String>,
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
    #[allow(clippy::missing_panics_doc)]
    pub async fn new<S: AsRef<str>>(node_name: S) -> Result<Node> {
        let zenoh_session = zenoh::open(zenoh::Config::default()).await?;
        let pubsub_data: Arc<Mutex<PubsubData>> = Arc::default();

        let node_name = node_name.as_ref().to_string();
        start_queriables(&zenoh_session, &node_name, pubsub_data.clone()).await?;

        info!(msg = "node_created", name = node_name);
        Ok(Node {
            node_name,
            zenoh_session,
            pubsub_data,
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
        let sub =
            Subscriber::new_from_session(&self.zenoh_session, topic, self.pubsub_data.clone())
                .await?;
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
        let sub = UntypedSubscriber::new_from_session(
            &self.zenoh_session,
            topic,
            &self.file_descriptor,
            self.pubsub_data.clone(),
        )
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
        let publisher =
            Publisher::new_from_session(&self.zenoh_session, topic, self.pubsub_data.clone())
                .await?;
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
            self.pubsub_data.clone(),
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

    /// Provides a full list of all nodes currently on the robotica network.
    ///
    /// # Errors
    /// As this uses the zenoh query function, any error returned to us by zenoh will be passed on
    /// to you.
    pub async fn list_nodes(&self) -> Result<HashSet<String>> {
        let recv = self
            .zenoh_session
            .get("robotica/node_names")
            .consolidation(ConsolidationMode::None)
            // .timeout(Duration::from_millis(500))
            .with(flume::unbounded())
            .await?;
        let mut result = HashSet::new();
        while let Ok(msg) = recv.recv_async().await {
            let bytes = msg.result()?.payload().to_bytes();
            let node_name = String::from_utf8_lossy(bytes.as_ref());
            result.insert(node_name.into());
        }

        Ok(result)
    }

    /// This function returns the list of subscribers a given node has currently active. We do so
    /// by sending a query to that specific node requesting the information.
    ///
    /// # Errors
    /// This will return an error if we fail to talk to the node over zenoh, if the data is
    /// incorrect, or simply if there's no response
    pub async fn list_nodes_subscribers(&self, node_name: &str) -> Result<Vec<SubscriberInfo>> {
        let recv = self
            .zenoh_session
            .get(format!("robotica/node/{node_name}/subscribers"))
            .with(flume::unbounded())
            .await?;
        let msg = recv.recv_async().await?;
        let bytes = msg.result()?.payload().to_bytes();
        let proto = SubscriberList::decode_length_delimited(bytes.as_ref())?;
        Ok(proto.subscribers)
    }

    /// This function returns the list of publishers a given node has currently active. We do so by
    /// sending a query to that specific node requesting the information.
    ///
    /// # Errors
    /// This will return an error if we fail to talk to the node over zenoh, if the data is
    /// incorrect, or simply if there's no response
    pub async fn list_nodes_publishers(&self, node_name: &str) -> Result<Vec<PublisherInfo>> {
        let recv = self
            .zenoh_session
            .get(format!("robotica/node/{node_name}/publishers"))
            .with(flume::unbounded())
            .await?;
        let msg = recv.recv_async().await?;
        let bytes = msg.result()?.payload().to_bytes();
        let proto = PublisherList::decode_length_delimited(bytes.as_ref())?;
        Ok(proto.publishers)
    }
}

async fn start_queriables(
    session: &Session,
    node_name: &str,
    pubsub_data: Arc<Mutex<PubsubData>>,
) -> Result {
    // Node name queryable
    let queryable = session
        .declare_queryable("robotica/node_names")
        .with(flume::unbounded())
        .await?;
    let node_name_clone = node_name.to_string();
    tokio::spawn(async move {
        if let Err(e) = node_name_queryable(queryable, node_name_clone).await {
            if !matches!(e, Error::Flume(flume::RecvError::Disconnected)) {
                tracing::error!(
                    msg = "error_node_queryable",
                    error = ?e,
                    "Error in node queryable, this is a bug in robotica",
                );
                panic!("Error in node queryable");
            }
        }
    });

    // Subscriber list
    let queryable = session
        .declare_queryable(format!("robotica/node/{node_name}/subscribers"))
        .with(flume::unbounded())
        .await?;
    let node_name_clone = node_name.to_string();
    let pubsub_data_clone = pubsub_data.clone();
    tokio::spawn(async move {
        if let Err(e) = subscribers_queryable(queryable, node_name_clone, pubsub_data_clone).await {
            if !matches!(e, Error::Flume(flume::RecvError::Disconnected)) {
                tracing::error!(
                    msg = "error_node_queryable",
                    error = ?e,
                    "Error in node queryable, this is a bug in robotica",
                );
                panic!("Error in node queryable");
            }
        }
    });

    // Pubisher list
    let queryable = session
        .declare_queryable(format!("robotica/node/{node_name}/publishers"))
        .with(flume::unbounded())
        .await?;
    let node_name_clone = node_name.to_string();
    tokio::spawn(async move {
        if let Err(e) = publishers_queryable(queryable, node_name_clone, pubsub_data).await {
            if !matches!(e, Error::Flume(flume::RecvError::Disconnected)) {
                tracing::error!(
                    msg = "error_node_queryable",
                    error = ?e,
                    "Error in node queryable, this is a bug in robotica",
                );
                panic!("Error in node queryable");
            }
        }
    });

    Ok(())
}

async fn node_name_queryable(
    queryable: zenoh::query::Queryable<flume::Receiver<zenoh::query::Query>>,
    node_name: String,
) -> Result {
    loop {
        let query = queryable.recv_async().await?;
        query.reply("robotica/node_names", &node_name).await?;
    }
}

async fn subscribers_queryable(
    queryable: zenoh::query::Queryable<flume::Receiver<zenoh::query::Query>>,
    node_name: String,
    pubsub_data: Arc<Mutex<PubsubData>>,
) -> Result {
    loop {
        let query = queryable.recv_async().await?;
        let msg = {
            let data = pubsub_data.lock().await;
            SubscriberList {
                subscribers: data
                    .subscribers
                    .iter()
                    .map(|name| SubscriberInfo { name: name.into() })
                    .collect(),
            }
        };
        query
            .reply(
                format!("robotica/node/{node_name}/subscribers"),
                &msg.encode_length_delimited_to_vec(),
            )
            .await?;
    }
}

async fn publishers_queryable(
    queryable: zenoh::query::Queryable<flume::Receiver<zenoh::query::Query>>,
    node_name: String,
    pubsub_data: Arc<Mutex<PubsubData>>,
) -> Result {
    loop {
        let query = queryable.recv_async().await?;
        let msg = {
            let data = pubsub_data.lock().await;
            PublisherList {
                publishers: data
                    .publishers
                    .iter()
                    .map(|name| PublisherInfo { name: name.into() })
                    .collect(),
            }
        };
        query
            .reply(
                format!("robotica/node/{node_name}/publishers"),
                &msg.encode_length_delimited_to_vec(),
            )
            .await?;
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
    /// Errors returned by Zenoh query replies
    #[error("zenoh query error: {0}")]
    ZenohQueryReply(#[from] zenoh::query::ReplyError),
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

impl From<&zenoh::query::ReplyError> for Error {
    fn from(e: &zenoh::query::ReplyError) -> Error {
        Error::ZenohQueryReply(e.clone())
    }
}

/// A type alias for results returned by functions in this library.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;
