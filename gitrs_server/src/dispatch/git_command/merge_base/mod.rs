mod is_ancestor;

use message::protocol::git_command::merge_base;
use state;
use types::DispatchFuture;

pub fn dispatch(
    connection_state: state::Connection,
    message: merge_base::Inbound,
) -> DispatchFuture {
    use self::merge_base::Inbound;

    match message {
        Inbound::IsAncestor { ancestor_sha, descendant_sha } =>
            is_ancestor::dispatch(connection_state, ancestor_sha, descendant_sha),
    }
}
