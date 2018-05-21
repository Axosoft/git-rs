mod parse;

use self::parse::{parse_bisect_step, BisectStep};
use error::protocol::SubhandlerError;
use futures::{future, Future};
use state;
use std;
use std::str;
use std::sync::{Arc, Mutex};
use tokio_process::CommandExt;
use types::DispatchFuture;
use util::git;
use util::transport::{read_message, send_message};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum InboundMessage {
    Bad,
    Good,
    Reset,
}

#[derive(Debug, Serialize)]
#[serde(tag = "reason")]
enum BisectError {
    RepoPathNotSet,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum OutboundMessage {
    BisectStep(BisectStep),
    Error(BisectError),
}

// See https://github.com/rust-lang/rfcs/issues/2407#issuecomment-385291238.
macro_rules! enclose {
    (($( $x:ident ),*) $y:expr) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub fn dispatch(connection_state: state::Connection, bad: String, good: String) -> DispatchFuture {
    use self::BisectError::RepoPathNotSet;
    use error::protocol::Error::Process;
    use error::protocol::ProcessError::{Encoding, Failed, Parsing};

    let repo_path = connection_state.repo_path.clone();
    let connection_state_mutex = Arc::new(Mutex::new(Some(connection_state)));
    Box::new(
        match repo_path {
            Some(repo_path) => future::ok((connection_state_mutex.clone(), repo_path)),
            None => future::err((
                connection_state_mutex.clone(),
                SubhandlerError::Subhandler(RepoPathNotSet),
            )),
        }.and_then(|(connection_state_mutex, repo_path)| {
            git::new_command_with_repo_path(&repo_path)
                .arg("bisect")
                .arg("start")
                .arg(bad)
                .arg(good)
                .arg("--")
                .output_async()
                .map_err(enclose! { (connection_state_mutex) |_| {
                    (
                        connection_state_mutex,
                        SubhandlerError::Shared(Process(Failed)),
                    )
                }})
                .map(enclose! { (connection_state_mutex) |output| {
                    (connection_state_mutex, output)
                } })
        })
            .and_then(|(connection_state_mutex, output)| {
                str::from_utf8(&output.stderr)
                    .map_err(|_| {
                        (
                            connection_state_mutex.clone(),
                            SubhandlerError::Shared(Process(Encoding)),
                        )
                    })
                    .and_then(|output| {
                        parse_bisect_step(output).map_err(|_| {
                            (
                                connection_state_mutex.clone(),
                                SubhandlerError::Shared(Process(Parsing)),
                            )
                        })
                    })
                    .map(|(_, bisect_step)| (connection_state_mutex.clone(), bisect_step))
            })
            .and_then(|(connection_state_mutex, bisect_step)| {
                let connection_state = connection_state_mutex.lock().unwrap().take().unwrap();
                send_message(connection_state, OutboundMessage::BisectStep(bisect_step))
                    .map(
                        enclose! { (connection_state_mutex) move |connection_state| {
                            let mut connection_state_guard = connection_state_mutex.lock().unwrap();
                            std::mem::replace(&mut *connection_state_guard, Some(connection_state));
                            connection_state_mutex.clone()
                        }},
                    )
                    .map_err(enclose! { (connection_state_mutex) |err| {
                        (connection_state_mutex, SubhandlerError::Shared(err))
                    }})
            })
            .map(|connection_state_mutex| {
                Arc::try_unwrap(connection_state_mutex)
                    .ok()
                    .unwrap()
                    .into_inner()
                    .unwrap()
                    .unwrap()
            })
            .or_else(|(connection_state_mutex, err)| match err {
                SubhandlerError::Shared(err) => Box::new(future::err(err)) as DispatchFuture,
                SubhandlerError::Subhandler(err) => {
                    let connection_state = Arc::try_unwrap(connection_state_mutex)
                        .ok()
                        .unwrap()
                        .into_inner()
                        .unwrap()
                        .unwrap();
                    Box::new(send_message(connection_state, OutboundMessage::Error(err)))
                        as DispatchFuture
                }
            }),
    )
}
