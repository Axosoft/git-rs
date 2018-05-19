mod parse;

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
pub enum ErrorReason {
    RepoPathNotSet,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum OutboundMessage {
    Success,
    Error(ErrorReason),
}

pub fn dispatch(connection_state: state::Connection, bad: String, good: String) -> DispatchFuture {
    use self::ErrorReason::RepoPathNotSet;
    use error::protocol::{Error, ProcessError::{Encoding, Failed}};

    match connection_state.repo_path.clone() {
        Some(repo_path) => Box::new(
            git::new_command_with_repo_path(&repo_path)
                .arg("bisect")
                .arg("start")
                .arg(bad)
                .arg(good)
                .output_async()
                .map_err(|_| Error::Process(Failed))
                .and_then(|output| match str::from_utf8(&output.stderr) {
                    Ok(output) => future::ok(String::from(output)),
                    Err(_) => future::err(Error::Process(Encoding)),
                })
                .and_then(|result| -> DispatchFuture {
                    println!("{:?}", result);
                    Box::new(future::ok(connection_state))
                })
        ),
        None => Box::new(send_message(
            connection_state,
            OutboundMessage::Error(RepoPathNotSet),
        )),
    }
}
