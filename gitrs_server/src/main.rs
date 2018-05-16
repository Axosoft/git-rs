extern crate bytes;
extern crate futures;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_process;
extern crate uuid;

mod dispatch;
mod error;
mod message;
mod state;
mod types;
mod util;

use dispatch::init_dispatch;
use futures::sync::mpsc::UnboundedSender as Sender;
use message::channel;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::prelude::*;
use uuid::Uuid;

pub fn main() {
    let state = Arc::new(Mutex::new(state::Shared::new()));
    let server_address = String::from("0.0.0.0:5134")
        .parse()
        .expect("Server address could not be parsed!");
    let listener =
        TcpListener::bind(&server_address).expect("TCP listener could not be bound to address!");
    let server = listener
        .incoming()
        .for_each(move |socket| {
            println!("accepted socket; addr={:?}", socket.peer_addr().unwrap());
            init_dispatch(state.clone(), socket);
            Ok(())
        })
        .map_err(|err| eprintln!("accept error = {:?}", err));

    println!("server running on {}", server_address);
    tokio::run(server);
}
