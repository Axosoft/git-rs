use super::message::protocol;
use super::message::protocol::{deserialize, serialize};
use error;

use bytes::Bytes;
use futures::future::Future;
use futures::{Sink, Stream};
use tokio::net::TcpStream;
use tokio_io::codec::length_delimited;

pub type Transport = length_delimited::Framed<TcpStream, Bytes>;

pub fn read_message(
    transport: Transport,
) -> impl Future<Item = (protocol::InboundMessage, Transport), Error = error::protocol::Error> {
    use error::protocol::{Error, TcpReceiveError};

    transport
        .into_future()
        .map_err(|_| Error::TcpReceive(TcpReceiveError::Io))
        .and_then(|(response, transport)| {
            let response = match response {
                Some(x) => x,
                None => unreachable!(),
            };
            println!("received message; message={:?}", response);
            deserialize(&response).map(|message| {
                println!("deserialized message; message={:?}", message);
                (message, transport)
            })
        })
}

#[allow(needless_pass_by_value)]
pub fn send_message(
    transport: Transport,
    message: protocol::OutboundMessage,
) -> impl Future<Item = Transport, Error = error::protocol::Error> {
    use error::protocol::{Error, TcpSendError};

    let message =
        serialize(&message).expect(&format!("Could not serialize message: {:?}", message));

    transport
        .send(message)
        .map_err(|_| Error::TcpSend(TcpSendError::Io))
}
