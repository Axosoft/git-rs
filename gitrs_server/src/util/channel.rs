use futures::sync::mpsc::{unbounded, UnboundedReceiver as Receiver, UnboundedSender as Sender};
use message::channel;

pub struct Channel {
    pub receiver: Receiver<channel::Message>,
    pub sender: Sender<channel::Message>,
}

impl Channel {
    pub fn new() -> Channel {
        Channel::from(unbounded())
    }
}

impl From<(Sender<channel::Message>, Receiver<channel::Message>)> for Channel {
    fn from((sender, receiver): (Sender<channel::Message>, Receiver<channel::Message>)) -> Channel {
        Channel { receiver, sender }
    }
}
