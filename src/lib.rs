use std::marker::PhantomData;
use zenoh::prelude::r#async::*;

mod publisher;
mod subscriber;
pub mod proto_types {
    include!(concat!(env!("OUT_DIR"), "/robotica.rs"));
}

pub use crate::publisher::Publisher;
pub use crate::subscriber::Subscriber;

pub struct Node {
    _node_name: String,
    zenoh_session: Session,
}

impl Node {
    #[allow(clippy::missing_errors_doc)]
    pub async fn new(node_name: String) -> Result<Node> {
        let zenoh_session = zenoh::open(config::default()).res().await?;
        Ok(Node {
            _node_name: node_name,
            zenoh_session,
        })
    }

    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub async fn subscribe<M: prost::Message + prost::Name + Default>(
        &self,
        topic: String,
    ) -> Result<Subscriber<'_, M>> {
        let subscriber = self
            .zenoh_session
            .declare_subscriber(&topic)
            .res()
            .await
            .unwrap();
        Ok(Subscriber {
            subscriber,
            _phantom: PhantomData,
        })
    }

    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub async fn publish<M: prost::Message + prost::Name>(
        &self,
        topic: String,
    ) -> Result<Publisher<'_, M>> {
        let publisher = self
            .zenoh_session
            .declare_publisher(topic)
            .res()
            .await
            .unwrap();
        Ok(Publisher {
            publisher,
            _phantom: PhantomData,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("zenoh error: {0}")]
    Zenoh(#[from] zenoh::Error),
    #[error("flume error: {0}")]
    Flume(#[from] flume::RecvError),
    #[error("subscriber expected message of type \"{expected}\", but received message of type \"{actual}\"")]
    MismatchedSubscriberType { expected: String, actual: String },
    #[error("error decoding protobuf: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;
