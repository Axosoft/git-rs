pub mod message;
mod transport;

use self::message::channel;
use self::message::protocol;
use self::transport::{read_message, send_message, Transport};
use SharedState;
use futures::future::Future;
use futures::sync::mpsc::{unbounded as channel, UnboundedReceiver as Receiver,
                          UnboundedSender as Sender};
use semver::Version;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpStream;
use uuid::Uuid;

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
