use crate::subscriber::Subscriber;
use zenoh::prelude::r#async::*;

mod subscriber;

pub struct Node {
    node_name: String,
    zenoh_session: Session,
}

impl Node {
    pub async fn new(node_name: String) -> Result<Node> {
        let zenoh_session = zenoh::open(config::default())
            .res()
            .await
            .map_err(Error::ZenohOpen)?;
        Ok(Node {
            node_name,
            zenoh_session,
        })
    }

    pub async fn subscribe<'a>(&'a self, topic: String) -> Result<Subscriber<'a>> {
        let subscriber = self
            .zenoh_session
            .declare_subscriber(&topic)
            .res()
            .await
            .unwrap();
        Ok(Subscriber { subscriber })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to open zenoh connection: {0}")]
    ZenohOpen(#[source] zenoh::Error),
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;
