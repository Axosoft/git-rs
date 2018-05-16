mod open_repo;
mod status;

use message::protocol::git_command;
use state;
use types::DispatchFuture;

pub fn dispatch(
    connection_state: state::Connection,
    message: git_command::Inbound,
) -> DispatchFuture {
    use self::git_command::Inbound;

    match message {
        Inbound::OpenRepo { path } => open_repo::dispatch(connection_state, path),
        Inbound::Status => status::dispatch(connection_state),
    }
}
