mod echo;

use self::echo::dispatch_echo_command;
use error;
use futures::Future;
use message::protocol::git_command;
use types::DispatchFuture;
use util::transport::Transport;

pub fn dispatch_git_command(
    transport: Transport,
    message: git_command::Inbound,
) -> DispatchFuture {
    use self::git_command::Inbound;
    match message {
        Inbound::Echo(echo_command) => dispatch_echo_command(transport, echo_command)
    }
}
