use bytes::{Bytes, BytesMut};
use error;
use futures::future::{self, Future};
use futures::{Sink, Stream};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;
use state;
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
    mut connection_state: state::Connection,
) -> impl Future<Item = (T, state::Connection), Error = error::protocol::Error>
where
    T: DeserializeOwned + Debug,
{
    use error::protocol::{Error, ProcessError, TcpReceiveError};

    future::result(
        connection_state
            .transport
            .take()
            .ok_or(Error::Process(ProcessError::Failed)),
    ).and_then(|transport| {
        transport
            .into_future()
            .map_err(|_| Error::TcpReceive(TcpReceiveError::Io))
    })
        .and_then(|(response, transport)| {
            let response = match response {
                Some(x) => x,
                None => unimplemented!(),
            };
            println!("received message; message={:?}", response);
            deserialize(response).map(|message| {
                println!("deserialized message; message={:?}", message);
                connection_state.transport = Some(transport);
                (message, connection_state)
            })
        })
}

#[allow(needless_pass_by_value)]
pub fn send_message<T>(
    mut connection_state: state::Connection,
    message: T,
) -> impl Future<Item = state::Connection, Error = error::protocol::Error>
where
    T: Serialize + Debug,
{
    use error::protocol::{Error, ProcessError, TcpSendError};

    let message =
        serialize(&message).expect(&format!("Could not serialize message: {:?}", message));

    future::result(
        connection_state
            .transport
            .take()
            .ok_or(Error::Process(ProcessError::Failed)),
    ).and_then(|transport| {
        transport
            .send(message)
            .map_err(|_| Error::TcpSend(TcpSendError::Io))
    })
        .map(|transport| {
            connection_state.transport = Some(transport);
            connection_state
        })
}
