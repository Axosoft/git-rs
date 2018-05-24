mod parse;

use self::parse::{parse_bisect, BisectFinish, BisectOutput, BisectStep};
use error::protocol::SubhandlerError;
use futures::future;
use futures::future::{loop_fn, Future, Loop};
use state;
use std::process;
use std::str;
use tokio_process::CommandExt;
use types::DispatchFuture;
use util::git;
use util::transport::{read_message, send_message};

// See https://github.com/rust-lang/rfcs/issues/2407#issuecomment-385291238.
macro_rules! enclose {
    (($($x:ident),*) $y:expr) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum InboundMessage {
    Bad,
    Good,
}

#[derive(Debug, Serialize)]
#[serde(tag = "reason")]
enum BisectError {
    RepoPathNotSet,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum OutboundMessage {
    Step(BisectStep),
    Error(BisectError),
    Finish(BisectFinish),
}

pub fn dispatch(connection_state: state::Connection, bad: String, good: String) -> DispatchFuture {
    use self::BisectError::RepoPathNotSet;
    use error::protocol::Error::{InboundMessage, Process};
    use error::protocol::InboundMessageError::Unexpected;
    use error::protocol::ProcessError::{Encoding, Failed, Parsing};

    Box::new(
        match connection_state.repo_path.clone() {
            Some(repo_path) => future::ok((repo_path, connection_state)),
            None => future::err((
                SubhandlerError::Subhandler(RepoPathNotSet),
                connection_state,
            )),
        }.and_then(move |(repo_path, connection_state)| {
            type CommandBuilder = Box<Fn() -> process::Command + Send>;

            let build_command_start: CommandBuilder = Box::new(enclose! { (repo_path) move || {
                let mut command = git::new_command_with_repo_path(&repo_path);
                command
                    .arg("bisect")
                    .arg("start")
                    .arg(bad.clone())
                    .arg(good.clone())
                    .arg("--");
                command
            }});

            let build_command_good = enclose! { (repo_path) move || {
                let mut command = git::new_command_with_repo_path(&repo_path);
                command.arg("bisect").arg("good");
                command
            }};

            let build_command_bad = enclose! { (repo_path) move || {
                let mut command = git::new_command_with_repo_path(&repo_path);
                command.arg("bisect").arg("bad");
                command
            }};

            let build_command_reset = enclose! { (repo_path) move || {
                let mut command = git::new_command_with_repo_path(&repo_path);
                command.arg("bisect").arg("reset");
                command
            }};

            loop_fn(
                (build_command_start, connection_state),
                move |(build_command, connection_state)| {
                    build_command()
                        .output_async()
                        .then(|result| match result {
                            Ok(output) => future::ok((output, connection_state)),
                            Err(_) => future::err((
                                SubhandlerError::Shared(Process(Failed)),
                                connection_state,
                            )),
                        })
                        .and_then(|(output, connection_state)| {
                            match str::from_utf8(&output.stdout) {
                                Ok(output) => future::ok((String::from(output), connection_state)),
                                Err(_) => future::err((
                                    SubhandlerError::Shared(Process(Encoding)),
                                    connection_state,
                                )),
                            }
                        })
                        .and_then(
                            enclose! { (build_command_bad, build_command_good, build_command_reset) move |(output, connection_state)| -> Box<Future<Item = _, Error = _> + Send> {
                                match parse_bisect(&output[..]) {
                                    Ok((_, output)) => match output {
                                        BisectOutput::Finish(bisect_finish) => Box::new(
                                            build_command_reset()
                                                .output_async()
                                                .then(|result| -> Box<Future<Item = _, Error = _> + Send> {
                                                    println!("Reset command finished!");
                                                    match result {
                                                        Ok(_) => Box::new(
                                                            send_message(connection_state, OutboundMessage::Finish(bisect_finish))
                                                                .map_err(|(err, connection_state)| (SubhandlerError::Shared(err), connection_state))
                                                        ),
                                                        Err(err) => Box::new(future::err((
                                                            SubhandlerError::Shared(Process(Failed)),
                                                            connection_state
                                                        )))
                                                    }
                                                })
                                                .map(Loop::Break)
                                            ),
                                        BisectOutput::Step(bisect_step) => Box::new(
                                            send_message(connection_state, OutboundMessage::Step(bisect_step))
                                                .map_err(|(err, connection_state)| (SubhandlerError::Shared(err), connection_state))
                                                .and_then(|connection_state| {
                                                    read_message(connection_state)
                                                        .map_err(|(err, connection_state)| (SubhandlerError::Shared(err), connection_state))
                                                })
                                                .and_then(|(message, connection_state)| {
                                                    match message {
                                                        self::InboundMessage::Bad => future::ok(Loop::Continue((
                                                            Box::new(build_command_bad) as CommandBuilder,
                                                            connection_state,
                                                        ))),
                                                        self::InboundMessage::Good => future::ok(Loop::Continue((
                                                            Box::new(build_command_good) as CommandBuilder,
                                                            connection_state,
                                                        ))),
                                                        _ => future::err((
                                                            SubhandlerError::Shared(InboundMessage(Unexpected)),
                                                            connection_state,
                                                        )),
                                                    }
                                                }),
                                        ),
                                    },
                                    Err(_) => Box::new(future::err((SubhandlerError::Shared(Process(Parsing)), connection_state))),
                                }
                            }},
                        )
                },
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
