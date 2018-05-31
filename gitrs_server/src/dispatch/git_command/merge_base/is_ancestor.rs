use futures::{future, Future};
use state;
use tokio_process::CommandExt;
use types::DispatchFuture;
use util::git;
use util::transport::send_message;

#[derive(Debug, Serialize)]
#[serde(tag = "reason")]
pub enum ErrorReason {
    AncestorMustBeASha,
    DescendantMustBeASha,
    RepoPathNotSet,
    ShaIsNotACommit,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum OutboundMessage {
    Success { is_ancestor: bool },
    Error(ErrorReason),
}

pub fn dispatch(
    connection_state: state::Connection,
    ancestor_sha: String,
    descendant_sha: String,
) -> DispatchFuture {
    use self::ErrorReason::{AncestorMustBeASha, DescendantMustBeASha, RepoPathNotSet,
                            ShaIsNotACommit};
    use error::protocol::{Error, ProcessError::Failed};

    if !git::verify_string_is_sha(&ancestor_sha) {
        return Box::new(send_message(
            connection_state,
            OutboundMessage::Error(AncestorMustBeASha),
        ));
    }

    if !git::verify_string_is_sha(&descendant_sha) {
        return Box::new(send_message(
            connection_state,
            OutboundMessage::Error(DescendantMustBeASha),
        ));
    }

    match connection_state.repo_path.clone() {
        Some(repo_path) => Box::new(
            git::new_command_with_repo_path(&repo_path)
                .arg("merge-base")
                .arg("--is-ancestor")
                .arg(ancestor_sha)
                .arg(descendant_sha)
                .output_async() // NOTE we should parse error messages someday
                .then(|result| match result {
                    Ok(output) => match output.status.code() {
                        Some(status) => future::ok((status, connection_state)),
                        None => future::err((Error::Process(Failed), connection_state)),
                    },
                    Err(_) => future::err((Error::Process(Failed), connection_state)),
                })
                .and_then(|(status, connection_state)| -> DispatchFuture {
                    Box::new(send_message(
                        connection_state,
                        if status == 0 || status == 1 {
                            OutboundMessage::Success { is_ancestor: status == 0 }
                        } else {
                            OutboundMessage::Error(ShaIsNotACommit)
                        }
                    ))
                }),
        ),
        None => Box::new(send_message(
            connection_state,
            OutboundMessage::Error(RepoPathNotSet),
        )),
    }
}
