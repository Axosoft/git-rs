use bytes::{Bytes, BytesMut};
use error;
use futures::future::Future;
use futures::{Sink, Stream};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;
use std::fmt::Debug;
use std::str;
use tokio::net::TcpStream;
use tokio_io::codec::length_delimited;

pub type Transport = length_delimited::Framed<TcpStream, Bytes>;

pub fn deserialize<T>(bytes: BytesMut) -> Result<T, error::protocol::Error>
where
    T: DeserializeOwned,
{
    str::from_utf8(&bytes)
        .map_err(error::protocol::Error::from)
        .and_then(|message| serde_json::from_str(&message).map_err(error::protocol::Error::from))
}

pub fn serialize<T>(message: &T) -> Result<Bytes, error::protocol::Error>
where
    T: Serialize,
{
    let message = serde_json::to_string(message).map_err(error::protocol::Error::from)?;
    Ok(Bytes::from(message.into_bytes()))
}

pub fn read_message<T>(
    transport: Transport,
) -> impl Future<Item = (T, Transport), Error = error::protocol::Error>
where
    T: DeserializeOwned + Debug,
{
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
            deserialize(response).map(|message| {
                println!("deserialized message; message={:?}", message);
                (message, transport)
            })
        })
}

#[allow(needless_pass_by_value)]
pub fn send_message<T>(
    transport: Transport,
    message: T,
) -> impl Future<Item = Transport, Error = error::protocol::Error>
where
    T: Serialize + Debug,
{
    use error::protocol::{Error, TcpSendError};

    let message =
        serialize(&message).expect(&format!("Could not serialize message: {:?}", message));

    transport
        .send(message)
        .map_err(|_| Error::TcpSend(TcpSendError::Io))
}
