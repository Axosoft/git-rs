mod echo;

use message::protocol::git_command;
use state;
use types::DispatchFuture;

pub fn dispatch(
    connection_state: state::Connection,
    message: git_command::Inbound,
) -> DispatchFuture {
    use self::git_command::Inbound;

    match message {
        Inbound::Echo(echo_command) => echo::dispatch(connection_state, echo_command),
    }
}
