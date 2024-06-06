use crate::publisher::Publisher;
use crate::subscriber::Subscriber;
use capnp::traits::{FromPointerReader, OwnedStruct};
use std::marker::PhantomData;
use zenoh::prelude::r#async::*;

mod publisher;
mod subscriber;

pub mod example_capnp {
    include!(concat!(env!("OUT_DIR"), "/example_capnp.rs"));
}

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
    pub async fn subscribe<'a, Message: OwnedStruct + FromPointerReader<'a> + Default>(
        &'a self,
        topic: String,
    ) -> Result<Subscriber<'a, Message>> {
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
    pub async fn publish(&self, topic: String) -> Result<Publisher<'_>> {
        let publisher = self
            .zenoh_session
            .declare_publisher(topic)
            .res()
            .await
            .unwrap();
        Ok(Publisher { publisher })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("zenoh error: {0}")]
    Zenoh(#[from] zenoh::Error),
    #[error("flume error: {0}")]
    Flume(#[from] flume::RecvError),
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;
