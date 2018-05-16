use futures::sync::mpsc::UnboundedSender as Sender;
use message::channel;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use util::channel::Channel;
use util::transport::Transport;
use uuid::Uuid;

#[derive(Default)]
pub struct Shared {
    channel_by_id: HashMap<Uuid, Sender<channel::Message>>,
}

impl Shared {
    pub fn new() -> Self {
        Default::default()
    }
}

pub struct Connection {
    channel: Channel,
    state: Arc<Mutex<Shared>>,
    pub transport: Option<Transport>,
    uuid: Uuid,
}

impl Connection {
    pub fn new(state: Arc<Mutex<Shared>>, transport: Transport) -> Self {
        let uuid = Uuid::new_v4();
        let channel = Channel::new();

        state
            .lock()
            .expect("Could not lock the shared state!")
            .channel_by_id
            .insert(uuid, channel.sender.clone());

        Connection {
            channel,
            state,
            transport: Some(transport),
            uuid,
        }
    }
}
