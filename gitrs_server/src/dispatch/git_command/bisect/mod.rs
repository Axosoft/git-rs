mod parse;

use self::parse::{parse_bisect_step, BisectStep};
use error::protocol::SubhandlerError;
use futures::{future, Future};
use state;
use std::process;
use std::str;
use tokio_process::CommandExt;
use types::DispatchFuture;
use util::future_context as context;
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

pub fn dispatch(connection_state: state::Connection, bad: String, good: String) -> DispatchFuture {
    use self::BisectError::RepoPathNotSet;
    use error::protocol::Error::Process;
    use error::protocol::ProcessError::{Encoding, Failed, Parsing};

    let repo_path = connection_state.repo_path.clone();
    Box::new(
        context::inject(
            connection_state,
            match repo_path {
                Some(repo_path) => Box::new(future::ok(repo_path)),
                None => Box::new(future::err(SubhandlerError::Subhandler(RepoPathNotSet))),
            },
        ).and_then(context::passthrough(move |repo_path: String| {
            Box::new(
                git::new_command_with_repo_path(&repo_path)
                    .arg("bisect")
                    .arg("start")
                    .arg(bad.clone())
                    .arg(good.clone())
                    .arg("--")
                    .output_async()
                    .map_err(|_| SubhandlerError::Shared(Process(Failed))),
            )
        }))
            .and_then(context::passthrough(|output: process::Output| {
                Box::new(future::result(
                    str::from_utf8(&output.stderr)
                        .map_err(|_| SubhandlerError::Shared(Process(Encoding)))
                        .and_then(|output| {
                            parse_bisect_step(output)
                                .map_err(|_| SubhandlerError::Shared(Process(Parsing)))
                        })
                        .map(|(_, bisect_step)| bisect_step),
                ))
            }))
            .and_then(context::map(|bisect_step, connection_state| {
                Box::new(
                    send_message(connection_state, OutboundMessage::BisectStep(bisect_step))
                        .map(|connection_state| ((), connection_state))
                        .map_err(|err| SubhandlerError::Shared(err)),
                )
            }))
            .map(context::dangerous::finish(|_, connection_state| {
                connection_state
            }))
            .or_else(context::dangerous::finish(|err, connection_state| match err {
                SubhandlerError::Shared(err) => Box::new(future::err(err)) as DispatchFuture,
                SubhandlerError::Subhandler(err) => {
                    Box::new(send_message(connection_state, OutboundMessage::Error(err)))
                        as DispatchFuture
                }
            })),
    )
}
