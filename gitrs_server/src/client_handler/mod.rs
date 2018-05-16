mod git_command;

use message;
use util::channel::Channel;
use util::transport::{read_message, send_message, Transport};

use self::git_command::dispatch_git_command;
use super::SharedState;

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

// TODO Move
type DispatchFuture = Box<Future<Item = Transport, Error = ::error::protocol::Error> + Send>;

// TODO Move
pub fn dispatch(transport: Transport, message: message::protocol::Inbound) -> DispatchFuture {
    use error::protocol::Error;
    use error::protocol::InboundMessageError::Unexpected;
    use message::protocol::Inbound;

    match message {
        Inbound::GitCommand(git_command) => dispatch_git_command(transport, git_command),
        _ => Box::new(future::err(Error::InboundMessage(Unexpected))),
    }
}

pub fn handle_client(state: Arc<Mutex<SharedState>>, socket: TcpStream) {
    use message::protocol::{Inbound, Outbound};
    let _client_handler = ClientHandler::new(state);
    let transport = Transport::new(socket);
    let connection = send_message(
        transport,
        Outbound::Hello {
            version: Version::new(0, 1, 0),
        },
    ).and_then(|transport| {
        println!("wrote hello message");
        read_validated_message!(Inbound::Hello, transport)
    })
        .and_then(|(_, transport)| send_message(transport, Outbound::GladToMeetYou))
        .and_then(|transport| {
            loop_fn(transport, |transport| {
                read_message(transport).and_then(
                    |(response, transport)| -> Box<
                        Future<Item = Loop<Transport, Transport>, Error = ::error::protocol::Error>
                            + Send,
                    > {
                        if let Inbound::Goodbye = response {
                            Box::new(future::ok(Loop::Break(transport)))
                        } else {
                            Box::new(dispatch(transport, response).map(Loop::Continue))
                        }
                    },
                )
            })
        })
        .and_then(|transport| send_message(transport, Outbound::Goodbye { error_code: None }))
        .and_then(|_| Ok(()))
        .map_err(|err| println!("error; err={:?}", err));

    tokio::spawn(connection);
}
