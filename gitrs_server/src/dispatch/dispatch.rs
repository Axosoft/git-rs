use super::git_command;
use futures::future;
use message;
use types::DispatchFuture;
use util::transport::Transport;

pub fn dispatch(transport: Transport, message: message::protocol::Inbound) -> DispatchFuture {
    use error::protocol::Error;
    use error::protocol::InboundMessageError::Unexpected;
    use message::protocol::Inbound;

    match message {
        Inbound::GitCommand(git_command) => git_command::dispatch(transport, git_command),
        _ => Box::new(future::err(Error::InboundMessage(Unexpected))),
    }
}
