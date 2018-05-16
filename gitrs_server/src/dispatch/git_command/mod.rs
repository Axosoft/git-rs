mod echo;

use error;
use futures::Future;
use message::protocol::git_command::Inbound;
use util::transport::Transport;

pub fn dispatch_git_command(
    transport: Transport,
    message: Inbound,
) -> Box<Future<Item = Transport, Error = error::protocol::Error> + Send> {
    unimplemented!()
}
