use super::git_command::dispatch_git_command;
use error;
use futures::future;
use message;
use types::DispatchFuture;
use util::transport::{read_message, send_message, Transport};

pub fn dispatch(transport: Transport, message: message::protocol::Inbound) -> DispatchFuture {
    use error::protocol::Error;
    use error::protocol::InboundMessageError::Unexpected;
    use message::protocol::Inbound;

    match message {
        Inbound::GitCommand(git_command) => dispatch_git_command(transport, git_command),
        _ => Box::new(future::err(Error::InboundMessage(Unexpected))),
    }
}
