pub mod message;

use self::message::channel;
use self::message::protocol;
use SharedState;
use bytes::{Bytes, BytesMut};
use error;
use futures::future::Future;
use futures::sync::mpsc::{unbounded as channel, UnboundedReceiver as Receiver,
                          UnboundedSender as Sender};
use futures::{future, Sink, Stream};
use semver::Version;
use serde_json;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpStream;
use tokio_io::codec::length_delimited;
use uuid::Uuid;

type Transport = length_delimited::Framed<TcpStream, Bytes>;

pub fn send(
    transport: Transport,
    message: &protocol::OutboundMessage,
) -> impl Future<Item = Transport, Error = error::protocol::Error> {
    use error::protocol::{Error, TcpSendError};

    let message = serde_json::to_string(message).unwrap();
    let message = Bytes::from(message.clone().into_bytes());
    transport
        .send(message)
        .map_err(|err| Error::TcpSend(TcpSendError::Io))
}

pub fn deserialize(bytes: &BytesMut) -> Result<protocol::InboundMessage, error::protocol::Error> {
    use error::protocol::{DeserializationError, Error};

    from_utf8(&bytes)
        .map_err(|_| Error::Deserialization(DeserializationError::Encoding))
        .and_then(|message| {
            serde_json::from_str(&message)
                .map_err(|serde_err| error::protocol::serde_json::to_error(serde_err))
        })
}

pub fn read1(
    transport: Transport,
) -> impl Future<Item = (protocol::InboundMessage, Transport), Error = error::protocol::Error> {
    use error::protocol::{Error, TcpReceiveError};

    transport
        .into_future()
        .map_err(|_| Error::TcpReceive(TcpReceiveError::Io))
        .and_then(|(response, transport)| {
            let response = response.unwrap();
            println!("received message; message={:?}", response);
            deserialize(&response).map(|message| (message, transport))
        })
}

struct ClientHandler {
    channel_receiver: Receiver<channel::Message>,
    channel_sender: Sender<channel::Message>,
    state: Arc<Mutex<SharedState>>,
    uuid: Uuid,
}

impl ClientHandler {
    fn new(state: Arc<Mutex<SharedState>>) -> Self {
        let uuid = Uuid::new_v4();
        let (sender, receiver) = channel();

        state
            .lock()
            .unwrap()
            .channel_by_id
            .insert(uuid, sender.clone());

        ClientHandler {
            channel_receiver: receiver,
            channel_sender: sender,
            state,
            uuid,
        }
    }
}

pub fn handle_client(state: Arc<Mutex<SharedState>>, socket: TcpStream) {
    use error::protocol::{Error, TcpReceiveError};

    let client_handler = ClientHandler::new(state);
    let transport = length_delimited::Framed::<_, Bytes>::new(socket);
    let connection = send(
        transport,
        &protocol::OutboundMessage::Hello {
            version: Version::new(0, 1, 0),
        },
    ).and_then(|transport| {
        println!("wrote hello message");
        read1(transport)
    })
        .map_err(|err| {
            println!("error; err={:?}", err);
        })
        .and_then(|(response, transport)| {
            println!("deserialized message; message={:?}", response);
            Ok(())
        });

    tokio::spawn(connection);
}
