use state;
use std::path::Path;
use types::DispatchFuture;
use util::transport::send_message;

#[derive(Debug, Serialize)]
#[serde(tag = "reason")]
pub enum ErrorReason {
    InvalidPath,
    MustBeAbsolutePath,
    IsNotRepo,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum OutboundMessage {
    Success,
    Error(ErrorReason),
}

pub fn dispatch(mut connection_state: state::Connection, workdir_path: String) -> DispatchFuture {
    use self::ErrorReason::{InvalidPath, IsNotRepo, MustBeAbsolutePath};
    let repo_path = Path::new(&workdir_path).join(".git");

    Box::new(if repo_path.is_relative() {
        send_message(connection_state, OutboundMessage::Error(MustBeAbsolutePath))
    } else {
        match repo_path.metadata() {
            Ok(metadata) => {
                if metadata.is_dir() {
                    connection_state.repo_path = Some(workdir_path);
                    send_message(connection_state, OutboundMessage::Success)
                } else {
                    send_message(connection_state, OutboundMessage::Error(IsNotRepo))
                }
            }
            Err(_) => send_message(connection_state, OutboundMessage::Error(InvalidPath)),
        }
    })
}
