mod bisect;
mod log;
mod merge_base;
mod open_repo;
mod status;

use message::protocol::git_command;
use state;
use types::DispatchFuture;

pub fn dispatch(
    connection_state: state::Connection,
    message: git_command::Inbound,
) -> DispatchFuture {
    use self::git_command::Inbound;

    match message {
        Inbound::Bisect { bad, good } => bisect::dispatch(connection_state, bad, good),
        Inbound::Log => log::dispatch(connection_state),
        Inbound::MergeBase(merge_base_arguments) =>
            merge_base::dispatch(connection_state, merge_base_arguments),
        Inbound::OpenRepo { path } => open_repo::dispatch(connection_state, path),
        Inbound::Status => status::dispatch(connection_state),
    }
}
