mod echo;
mod open_repo;

use message::protocol::git_command;
use state;
use types::DispatchFuture;

pub fn dispatch(
    connection_state: state::Connection,
    message: git_command::Inbound,
) -> DispatchFuture {
    use self::git_command::Inbound;

    match message {
        Inbound::Echo { input } => echo::dispatch(connection_state, input),
        Inbound::OpenRepo { path } => open_repo::dispatch(connection_state, path),
    }
}
