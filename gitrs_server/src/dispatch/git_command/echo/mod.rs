mod message;

use self::message as internal_echo_message;
use message::protocol::git_command::echo as shared_echo_message;
use util::transport::{send_message, Transport};

use error;
use futures::Future;
use std::process::Command;
use tokio_core::reactor::Core;
use tokio_process::CommandExt;

pub fn dispatch_echo_command(transport: Transport, message: shared_echo_message::Inbound) -> () {
    // impl Future<Item = (internal_echo_message::Outbound, Transport), Error = error::protocol::Error> {
    unimplemented!()
    // use error::protocol::{Error, TcpReceiveError};

    // let mut core = Core::new().unwrap();

    // let output = Command::new("echo").arg("hello").arg("world")
    //                     .output_async(&core.handle());

    // let message =
    // serialize(&message).expect(&format!("Could not serialize message: {:?}", message));

    // transport
    //     .send(message)
}
