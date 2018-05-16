use error;
use futures::future::Future;
use state;

pub type DispatchFuture =
    Box<Future<Item = state::Connection, Error = error::protocol::Error> + Send>;
