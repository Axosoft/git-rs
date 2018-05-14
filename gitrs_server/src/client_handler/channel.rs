use super::message;
use futures::sync::mpsc::{unbounded, UnboundedReceiver as Receiver, UnboundedSender as Sender};

pub struct Channel {
    pub receiver: Receiver<message::channel::Message>,
    pub sender: Sender<message::channel::Message>,
}

impl Channel {
    pub fn new() -> Channel {
        Channel::from(unbounded())
    }
}

impl
    From<(
        Sender<message::channel::Message>,
        Receiver<message::channel::Message>,
    )> for Channel
{
    fn from(
        (sender, receiver): (
            Sender<message::channel::Message>,
            Receiver<message::channel::Message>,
        ),
    ) -> Channel {
        Channel { receiver, sender }
    }
}
