mod channel;
pub mod message;
mod transport;

use self::channel::Channel;
use self::message::protocol;
use self::transport::{read_message, send_message, Transport};
use SharedState;
use futures::future;
use futures::future::{loop_fn, Future, Loop};
use semver::Version;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpStream;
use uuid::Uuid;

macro_rules! read_validated_message {
    ($messagePattern:pat, $transport:expr) => {
        read_message($transport).and_then(|(response, transport)| {
            use error::protocol::{Error, InboundMessageError};

            match response {
                $messagePattern => Ok((response, transport)),
                _ => Err(Error::InboundMessage(InboundMessageError::Unexpected)),
            }
        })
    };
}

struct ClientHandler {
    channel: Channel,
    state: Arc<Mutex<SharedState>>,
    uuid: Uuid,
}

impl ClientHandler {
    fn new(state: Arc<Mutex<SharedState>>) -> Self {
        let uuid = Uuid::new_v4();
        let channel = Channel::new();

        state
            .lock()
            .expect("Could not lock the shared state!")
            .channel_by_id
            .insert(uuid, channel.sender.clone());

        ClientHandler {
            channel,
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
        read_validated_message!(protocol::InboundMessage::Hello, transport)
    })
        .and_then(|(_, transport)| {
            send_message(transport, protocol::OutboundMessage::GladToMeetYou)
        })
        .and_then(|transport| {
            loop_fn(transport, |transport| {
                read_message(transport).and_then(|(response, transport)| {
                    let (cmd_future, continue_looping) = match response {
                        protocol::InboundMessage::Goodbye => {
                            (Box::new(future::ok(transport)), false)
                        }
                        protocol::InboundMessage::RunGitCommand => {
                            (Box::new(future::ok(transport)), true)
                        }
                        _ => (Box::new(future::ok(transport)), true),
                    };

                    Box::new(cmd_future.and_then(move |transport| {
                        if continue_looping {
                            Ok(Loop::Continue(transport))
                        } else {
                            Ok(Loop::Break(transport))
                        }
                    }))
                })
            })
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
