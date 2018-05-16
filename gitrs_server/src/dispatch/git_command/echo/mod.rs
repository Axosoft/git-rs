mod message;

use self::message as internal_echo_message;

use error;
use futures::Future;
use message::protocol::git_command::echo as shared_echo_message;
use std::process::Command;
use std::str;
use tokio_process::CommandExt;
use types::DispatchFuture;
use util::transport::{send_message, Transport};

pub fn dispatch_echo_command(
    transport: Transport,
    message: shared_echo_message::Inbound,
) -> DispatchFuture {
    use error::protocol::{Error, ProcessError::Failed};

    Box::new(
        Command::new("echo")
            .arg(&message.input)
            .output_async()
            .map_err(|_| Error::Process(Failed))
            .map(|output| {
                println!("{}", str::from_utf8(&output.stdout).unwrap());
                transport
            }),
    )
}
