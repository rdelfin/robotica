use crate::Result;
use capnp::{
    message::{Reader, ReaderOptions},
    serialize::OwnedSegments,
    traits::{FromPointerReader, OwnedStruct},
};
use std::{io::BufReader, marker::PhantomData};
use zenoh::prelude::r#async::*;
use zenoh::subscriber::FlumeSubscriber;

pub struct Subscriber<'a, Message: OwnedStruct + FromPointerReader<'a> + Default> {
    pub subscriber: FlumeSubscriber<'a>,
    pub _phantom: PhantomData<Message>,
}

impl<'a, Message: OwnedStruct + FromPointerReader<'a> + Default> Subscriber<'a, Message> {
    pub async fn recv(&self) -> Result<OwnedSegments> {
        let sample = self.subscriber.recv_async().await?;
        let payload_reader = sample.value.payload.reader();
        let buf_reader = BufReader::new(payload_reader);
        let reader =
            capnp::serialize_packed::read_message(buf_reader, capnp::message::ReaderOptions::new())
                .unwrap();
        Ok(reader.into_segments())
    }
}
