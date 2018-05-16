mod dispatch;
mod git_command;

use self::dispatch::dispatch;
use futures::future;
use futures::future::{loop_fn, Future, Loop};
use message;
use semver::Version;
use ::state;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::net::TcpStream;
use util::transport::{read_message, send_message, Transport};

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

pub fn init_dispatch(state: Arc<Mutex<state::Shared>>, socket: TcpStream) {
    use message::protocol::{Inbound, Outbound};
    let connection_state = state::Connection::new(state);
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
