mod echo;

use error;
use futures::Future;
use message::protocol::git_command;
use types::DispatchFuture;
use util::transport::Transport;

pub fn dispatch(transport: Transport, message: git_command::Inbound) -> DispatchFuture {
    use self::git_command::Inbound;

    match message {
        Inbound::Echo(echo_command) => echo::dispatch(transport, echo_command),
    }
}
