mod parse;

use error::protocol::SubhandlerError;
use futures::{future, Future};
use state;
use std::str;
use tokio_process::CommandExt;
use types::DispatchFuture;
use util::git;
use util::transport::{read_message, send_message};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum InboundMessage {
    Bad,
    Good,
    Reset,
}

#[derive(Debug, Serialize)]
#[serde(tag = "reason")]
pub enum BisectError {
    RepoPathNotSet,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum OutboundMessage {
    Success,
    Error(BisectError),
}

pub fn dispatch(connection_state: state::Connection, bad: String, good: String) -> DispatchFuture {
    use self::BisectError::RepoPathNotSet;
    use error::protocol::{Error, ProcessError::{Encoding, Failed}};

    Box::new(
        match connection_state.repo_path.clone() {
            Some(repo_path) => future::ok((repo_path, connection_state)),
            None => future::err((
                SubhandlerError::Subhandler(RepoPathNotSet),
                connection_state,
            )),
        }.and_then(|(repo_path, connection_state)| {
            Box::new(
                git::new_command_with_repo_path(&repo_path)
                    .arg("bisect")
                    .arg("start")
                    .arg(bad)
                    .arg(good)
                    .output_async()
                    .map_err(|_| SubhandlerError::Shared(Error::Process(Failed)))
                    .and_then(|output| match str::from_utf8(&output.stderr) {
                        Ok(output) => future::ok(String::from(output)),
                        Err(_) => future::err(SubhandlerError::Shared(Error::Process(Encoding))),
                    })
                    .then(|result| match result {
                        Ok(_result) => future::ok(connection_state),
                        Err(err) => future::err((err, connection_state)),
                    }),
            )
        })
            .or_else(|(err, connection_state)| match err {
                SubhandlerError::Shared(err) => {
                    Box::new(future::err((err, connection_state))) as DispatchFuture
                }
                SubhandlerError::Subhandler(err) => {
                    Box::new(send_message(connection_state, OutboundMessage::Error(err)))
                        as DispatchFuture
                }
            }),
    )
}
