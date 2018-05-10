extern crate bytes;
extern crate futures;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate tokio_io;
extern crate uuid;

mod client_handler;
mod error;

use client_handler::handle_client;
use client_handler::message::channel;
use futures::sync::mpsc::UnboundedSender as Sender;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::prelude::*;
use uuid::Uuid;

pub struct SharedState {
    channel_by_id: HashMap<Uuid, Sender<channel::Message>>,
}

impl SharedState {
    pub fn new() -> Self {
        SharedState {
            channel_by_id: HashMap::new(),
        }
    }
}

pub fn main() {
    let state = Arc::new(Mutex::new(SharedState::new()));
    let server_address = String::from("0.0.0.0:5134").parse().unwrap();
    let listener = TcpListener::bind(&server_address).unwrap();
    let server = listener
        .incoming()
        .for_each(move |socket| {
            println!("accepted socket; addr={:?}", socket.peer_addr().unwrap());
            handle_client(state.clone(), socket);
            Ok(())
        })
        .map_err(|err| {
            eprintln!("accept error = {:?}", err);
        });

    println!("server running on {}", server_address);
    tokio::run(server);
}
