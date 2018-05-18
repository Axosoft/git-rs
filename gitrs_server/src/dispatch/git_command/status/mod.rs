mod status_entry;

use futures::{future, Future};
use self::status_entry::{parse_status_entries, StatusEntry};
use state;
use std::process::Command;
use std::str;
use tokio_process::CommandExt;
use types::DispatchFuture;
use util::transport::send_message;

#[derive(Debug, Serialize)]
#[serde(tag = "reason")]
pub enum ErrorReason {
    RepoPathNotSet,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum OutboundMessage {
    Success { result: Vec<StatusEntry> },
    Error(ErrorReason),
}

pub fn dispatch(connection_state: state::Connection) -> DispatchFuture {
    use self::ErrorReason::RepoPathNotSet;
    use error::protocol::{Error, ProcessError::{Encoding, Failed}};

    match connection_state.repo_path.clone() {
        Some(repo_path) => Box::new(
            Command::new("git")
                .arg("--no-pager")
                .arg("status")
                .arg("--porcelain=v2")
                .arg("--untracked-files")
                .current_dir(&repo_path)
                .output_async()
                .map_err(|_| Error::Process(Failed))
                .and_then(|output| match str::from_utf8(&output.stdout) {
                    Ok(output) => future::ok(String::from(output)),
                    Err(_) => future::err(Error::Process(Encoding)),
                })
                .and_then(|result| -> DispatchFuture {
                    let mut result = String::from(result);
                    result.push('\0'); // NOTE oh no :(
                    match parse_status_entries(&result) {
                        Ok((_, result)) => Box::new(send_message(connection_state, OutboundMessage::Success { result })),
                        Err(err) => Box::new(future::err(Error::Process(Encoding))), // TODO Change me to something else please
                    }
                }),
        ),
        None => Box::new(send_message(
            connection_state,
            OutboundMessage::Error(RepoPathNotSet),
        )),
    }
}
