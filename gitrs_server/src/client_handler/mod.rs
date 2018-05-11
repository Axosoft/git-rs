pub mod message;

use self::message::channel;
use self::message::protocol;
use SharedState;
use bytes::{Bytes, BytesMut};
use error;
use futures::future::Future;
use futures::sync::mpsc::{unbounded as channel, UnboundedReceiver as Receiver,
                          UnboundedSender as Sender};
use futures::{Sink, Stream};
use semver::Version;
use serde_json;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpStream;
use tokio_io::codec::length_delimited;
use uuid::Uuid;

type Transport = length_delimited::Framed<TcpStream, Bytes>;

#[allow(needless_pass_by_value)]
pub fn send_message(
    transport: Transport,
    message: protocol::OutboundMessage,
) -> impl Future<Item = Transport, Error = error::protocol::Error> {
    use error::protocol::{Error, TcpSendError};

    let message = serde_json::to_string(&message)
        .expect(&format!("Could not serialize message: {:?}", message));
    let message = Bytes::from(message.into_bytes());
    transport
        .send(message)
        .map_err(|_| Error::TcpSend(TcpSendError::Io))
}

pub fn deserialize(bytes: &BytesMut) -> Result<protocol::InboundMessage, error::protocol::Error> {
    use error::protocol::{DeserializationError, Error};

    from_utf8(&bytes)
        .map_err(|_| Error::Deserialization(DeserializationError::Encoding))
        .and_then(|message| {
            serde_json::from_str(&message)
                .map_err(|serde_err| error::protocol::serde_json::to_error(&serde_err))
        })
}

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
            .expect("Could not lock the shared state!")
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
    let _client_handler = ClientHandler::new(state);
    let transport = Transport::new(socket);
    let connection = send_message(
        transport,
        protocol::OutboundMessage::Hello {
            version: Version::new(0, 1, 0),
        },
    ).and_then(|transport| {
        println!("wrote hello message");
        read_message(transport)
    })
        .and_then(|(response, transport)| {
            use error::protocol::{Error, InboundMessageError};

            match response {
                protocol::InboundMessage::Hello => Ok(transport),
                _ => Err(Error::InboundMessage(InboundMessageError::Unexpected)),
            }
        })
        .and_then(|transport| send_message(transport, protocol::OutboundMessage::GladToMeetYou))
        .and_then(read_message)
        .and_then(|(response, transport)| {
            use error::protocol::{Error, InboundMessageError};

            match response {
                protocol::InboundMessage::Goodbye => Ok(transport),
                _ => Err(Error::InboundMessage(InboundMessageError::Unexpected)),
            }
        })
        .and_then(|transport| {
            send_message(
                transport,
                protocol::OutboundMessage::Goodbye { error_code: None },
            )
        })
        .and_then(|_| Ok(()))
        .map_err(|err| println!("error; err={:?}", err));

    tokio::spawn(connection);
}
