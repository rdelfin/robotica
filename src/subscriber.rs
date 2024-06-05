use futures::{stream::Stream, Future};
use zenoh::prelude::r#async::*;

pub struct Subscriber<'a> {
    pub subscriber: zenoh::subscriber::FlumeSubscriber<'a>,
}

impl<'a> Stream for Subscriber<'a> {
    type Item = zenoh::protocol::Message;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.subscriber.recv_async().poll(cx)
    }
}
