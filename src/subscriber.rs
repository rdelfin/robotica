use crate::{proto_types::Header, Error, Result};
use prost::Message;
use std::marker::PhantomData;
use zenoh::{prelude::r#async::*, subscriber::FlumeSubscriber};

pub struct Subscriber<'a, M: prost::Message + prost::Name + Default> {
    pub subscriber: FlumeSubscriber<'a>,
    pub _phantom: PhantomData<M>,
}

impl<'a, M: prost::Message + prost::Name + Default> Subscriber<'a, M> {
    #[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
    pub async fn recv(&self) -> Result<ReceivedMessage<M>> {
        let sample = self.subscriber.recv_async().await?;
        let bytes = sample.value.payload.contiguous();
        let mut byte_ref = bytes.as_ref();
        let header =
            Header::decode_length_delimited(&mut byte_ref).expect("failed to decode header");
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
