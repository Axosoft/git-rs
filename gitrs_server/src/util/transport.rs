use bytes::{Bytes, BytesMut};
use config;
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
use types::DispatchFuture;

pub type Transport = length_delimited::Framed<TcpStream, Bytes>;

pub fn deserialize<T>(bytes: &BytesMut) -> Result<T, error::protocol::Error>
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

pub fn read_message<T: Send + 'static>(
    mut connection_state: state::Connection,
) -> Box<
    Future<Item = (T, state::Connection), Error = (error::protocol::Error, state::Connection)>
        + Send,
>
where
    T: DeserializeOwned + Debug,
{
    use error::protocol::{Error, ProcessError, TcpReceiveError};

    match connection_state.transport.take() {
        Some(transport) => Box::new(transport.into_future().then(|result| match result {
            Ok((response, transport)) => {
                let response = match response {
                    Some(x) => x,
                    None => unimplemented!(),
                };
                debug!({
                    println!("received message; message={:?}", response);
                });
                connection_state.transport = Some(transport);
                match deserialize(&response) {
                    Ok(message) => {
                        debug!({
                            println!("deserialized message; message={:?}", message);
                        });
                        future::ok((message, connection_state))
                    }
                    Err(err) => future::err((err, connection_state)),
                }
            }
            Err((_, transport)) => {
                connection_state.transport = Some(transport);
                future::err((Error::TcpReceive(TcpReceiveError::Io), connection_state))
            }
        })),
        None => Box::new(future::err((
            Error::Process(ProcessError::Failed),
            connection_state,
        ))),
    }
}

#[allow(needless_pass_by_value)]
pub fn send_message<T>(mut connection_state: state::Connection, message: T) -> DispatchFuture
where
    T: Serialize + Debug,
{
    use error::protocol::{Error, ProcessError, TcpSendError};

    let message =
        serialize(&message).expect(&format!("Could not serialize message: {:?}", message));

    match connection_state.transport.take() {
        Some(transport) => Box::new(transport.send(message).then(|result| match result {
            Ok(transport) => {
                connection_state.transport = Some(transport);
                future::ok(connection_state)
            }
            Err(_) => future::err((Error::TcpSend(TcpSendError::Io), connection_state)),
        })),
        None => Box::new(future::err((
            Error::Process(ProcessError::Failed),
            connection_state,
        ))),
    }
}
