use error;
use futures::future::Future;
use util::transport::Transport;

pub type DispatchFuture = Box<Future<Item = Transport, Error = error::protocol::Error> + Send>;
