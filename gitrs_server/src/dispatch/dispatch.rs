use super::git_command;
use futures::future;
use message;
use state;
use types::DispatchFuture;

pub fn dispatch(connection_state: state::Connection, message: message::protocol::Inbound) -> DispatchFuture {
    use error::protocol::Error;
    use error::protocol::InboundMessageError::Unexpected;
    use message::protocol::Inbound;

    match message {
        Inbound::GitCommand(git_command) => git_command::dispatch(connection_state, git_command),
        _ => Box::new(future::err((Error::InboundMessage(Unexpected), connection_state))),
    }
}
