pub mod message;

use super::SharedState;
use futures::future::{AndThen, Future};
use futures::sync::mpsc::{unbounded as channel, UnboundedReceiver as Receiver,
                          UnboundedSender as Sender};
use self::message::channel;
use self::message::protocol;
use semver::Version;
use serde_json;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::io;
use tokio::net::TcpStream;
use uuid::Uuid;

struct ClientHandler {
    channel_receiver: Receiver<channel::Message>,
    channel_sender: Sender<channel::Message>,
    state: Arc<Mutex<SharedState>>,
    socket: TcpStream,
    uuid: Uuid,
}

impl ClientHandler {
    fn new(state: Arc<Mutex<SharedState>>, socket: TcpStream) -> Self {
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
            socket,
            uuid,
        }
    }
}

pub fn handle_client(state: Arc<Mutex<SharedState>>, socket: TcpStream) {
    let client_handler = ClientHandler::new(state, socket);
    let hello_message = protocol::OutboundMessage::Hello {
        version: Version::new(0, 1, 0),
    };
    let hello_message = serde_json::to_string(&hello_message).unwrap();
    let connection = io::write_all(client_handler.socket, hello_message).then(|result| {
        println!("wrote message; success={:?}", result.is_ok());
        Ok(())
    });

    tokio::spawn(connection);
}
