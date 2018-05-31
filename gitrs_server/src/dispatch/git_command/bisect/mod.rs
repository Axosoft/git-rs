mod parse;

use self::parse::{parse_bisect, BisectFinish, BisectOutput, BisectReachedMergeBase, BisectStep,
                  BisectVisualize};
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
    Reset,
    Visualize,
}

#[derive(Debug, Serialize)]
#[serde(tag = "reason")]
enum BisectError {
    AlreadyBisecting,
    RepoPathNotSet,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum OutboundMessage {
    Error(BisectError),
    Finish(BisectFinish),
    ReachedMergeBase(BisectReachedMergeBase),
    Step(BisectStep),
    Visualize(BisectVisualize),
    Success,
}

type CommandBuilder = Box<Fn() -> process::Command + Send>;

fn verify_repo_path(repo_path: Option<String>) -> Result<String, SubhandlerError<BisectError>> {
    use self::BisectError::RepoPathNotSet;

    match repo_path {
        Some(repo_path) => Ok(repo_path),
        None => Err(SubhandlerError::Subhandler(RepoPathNotSet)),
    }
}

fn run_command(
    build_command: CommandBuilder,
) -> impl Future<Item = String, Error = SubhandlerError<BisectError>> {
    use error::protocol::Error::Process;
    use error::protocol::ProcessError::{Encoding, Failed};

    build_command()
        .output_async()
        .map_err(|_| SubhandlerError::Shared(Process(Failed)))
        .and_then(|output| {
            let output = if output.stdout.is_empty() {
                output.stderr
            } else {
                output.stdout
            };
            str::from_utf8(&output)
                .map(String::from)
                .map_err(|_| SubhandlerError::Shared(Process(Encoding)))
        })
}

type FinishBisectError = (SubhandlerError<BisectError>, state::Connection);

fn finish_bisect(
    bisect_finish: BisectFinish,
    build_command_reset: CommandBuilder,
    connection_state: state::Connection,
) -> impl Future<Item = state::Connection, Error = FinishBisectError> {
    run_command(build_command_reset).then(|result| -> Box<Future<Item = _, Error = _> + Send> {
        match result {
            Ok(_) => Box::new(
                send_message(connection_state, OutboundMessage::Finish(bisect_finish)).map_err(
                    |(err, connection_state)| (SubhandlerError::Shared(err), connection_state),
                ),
            ),
            Err(err) => Box::new(future::err((err, connection_state))),
        }
    })
}

fn handle_errors(
    (err, connection_state): (SubhandlerError<BisectError>, state::Connection),
) -> DispatchFuture {
    match err {
        SubhandlerError::Shared(err) => Box::new(future::err((err, connection_state))),
        SubhandlerError::Subhandler(err) => {
            Box::new(send_message(connection_state, OutboundMessage::Error(err)))
        }
    }
}

type LoopFuture = Box<
    Future<
            Item = Loop<state::Connection, (CommandBuilder, state::Connection)>,
            Error = (SubhandlerError<BisectError>, state::Connection),
        >
        + Send,
>;

fn build_bisect_step_handler(
    repo_path: String,
) -> impl FnOnce((String, state::Connection)) -> LoopFuture {
    use error::protocol::Error::Process;
    use error::protocol::ProcessError::Parsing;

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

    let build_command_visualize = enclose! { (repo_path) move || {
        let mut command = git::new_command_with_repo_path(&repo_path);
        command.arg("bisect").arg("visualize").arg("--format=sha %H");
        command
    }};

    enclose! { (build_command_bad, build_command_good, build_command_reset, build_command_visualize)
        move |(output, connection_state): (String, state::Connection)| -> LoopFuture {
            println!("{}", output);
            match parse_bisect(&output[..]) {
                Ok((_, output)) => match output {
                    BisectOutput::Finish(bisect_finish) => Box::new(
                        finish_bisect(
                            bisect_finish,
                            Box::new(build_command_reset),
                            connection_state
                        )
                            .map(Loop::Break)
                    ),
                    output => {
                        let message = match output {
                            BisectOutput::Step(bisect_step) => OutboundMessage::Step(bisect_step),
                            BisectOutput::ReachedMergeBase(bisect_reached_merge_base) => {
                                OutboundMessage::ReachedMergeBase(bisect_reached_merge_base)
                            },
                            BisectOutput::Visualize(bisect_visualize) => {
                                OutboundMessage::Visualize(bisect_visualize)
                            },
                            BisectOutput::Finish(_) => unreachable!("`Finish(_)` should have been handled by the outer `match`."),
                        };

                        Box::new(
                            send_message(connection_state, message)
                                .map_err(|(err, connection_state)| (
                                    SubhandlerError::Shared(err),
                                    connection_state
                                ))
                                .and_then(|connection_state| {
                                    read_message(connection_state)
                                        .map_err(|(err, connection_state)| (
                                            SubhandlerError::Shared(err),
                                            connection_state
                                        ))
                                })
                                .and_then(|(message, connection_state)| -> LoopFuture {
                                    match message {
                                        InboundMessage::Bad => Box::new(
                                            future::ok(
                                                Loop::Continue((
                                                    Box::new(build_command_bad) as CommandBuilder,
                                                    connection_state
                                                ))
                                            )
                                        ),
                                        InboundMessage::Good => Box::new(
                                            future::ok(
                                                Loop::Continue((
                                                    Box::new(build_command_good) as CommandBuilder,
                                                    connection_state
                                                ))
                                            )
                                        ),
                                        InboundMessage::Reset => Box::new(
                                            run_command(Box::new(build_command_reset))
                                                .then(|result| {
                                                    match result {
                                                        Ok(_) => Box::new(
                                                            send_message(
                                                                connection_state,
                                                                OutboundMessage::Success
                                                            )
                                                                .map_err(|(err, connection_state)| (
                                                                    SubhandlerError::Shared(err),
                                                                    connection_state
                                                                ))
                                                                .map(Loop::Break)
                                                        ),
                                                        Err(err) => Box::new(
                                                            future::err((err, connection_state))
                                                        ) as LoopFuture
                                                    }
                                                })
                                        ),
                                        InboundMessage::Visualize => Box::new(
                                            future::ok(
                                                Loop::Continue((
                                                    Box::new(
                                                        build_command_visualize
                                                    ) as CommandBuilder,
                                                    connection_state
                                                ))
                                            )
                                        ),
                                    }
                                })
                        )
                    },
                },
                Err(_) => Box::new(
                    future::err((SubhandlerError::Shared(Process(Parsing)), connection_state))
                ),
            }
        }
    }
}

pub fn dispatch(connection_state: state::Connection, bad: String, good: String) -> DispatchFuture {
    use self::BisectError::AlreadyBisecting;
    use error::protocol::Error::Process;
    use error::protocol::ProcessError::Failed;

    Box::new(
        future::result(match verify_repo_path(connection_state.repo_path.clone()) {
            Ok(repo_path) => Ok((repo_path, connection_state)),
            Err(err) => Err((err, connection_state)),
        }).and_then(move |(repo_path, connection_state)| {
            git::new_command_with_repo_path(&repo_path)
                .arg("bisect")
                .arg("log")
                .output_async()
                .then(|output| match output {
                    Ok(output) => match output.status.code() {
                        Some(status) => if status == 0 {
                            Box::new(future::err((
                                SubhandlerError::Subhandler(AlreadyBisecting),
                                connection_state,
                            )))
                        } else {
                            Box::new(future::ok((repo_path, connection_state)))
                        },
                        None => Box::new(future::err((
                            SubhandlerError::Shared(Process(Failed)),
                            connection_state,
                        ))),
                    },
                    Err(_) => Box::new(future::err((
                        SubhandlerError::Shared(Process(Failed)),
                        connection_state,
                    ))),
                })
        })
            .and_then(move |(repo_path, connection_state)| {
                let build_command_start: CommandBuilder =
                    Box::new(enclose! { (repo_path) move || {
                        let mut command = git::new_command_with_repo_path(&repo_path);
                        command
                            .arg("bisect")
                            .arg("start")
                            .arg(bad.clone())
                            .arg(good.clone())
                            .arg("--");
                        command
                    }});

                loop_fn(
                    (build_command_start, connection_state),
                    enclose! { (repo_path)
                        move |(build_command, connection_state)| {
                            run_command(build_command)
                                .then(|result| match result {
                                    Ok(output) => future::ok((output, connection_state)),
                                    Err(err) => future::err((err, connection_state)),
                                })
                                .and_then(build_bisect_step_handler(repo_path.clone()))
                        }
                    },
                )
            })
            .or_else(handle_errors),
    )
}
