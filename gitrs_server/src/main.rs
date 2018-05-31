extern crate bytes;
extern crate clap;
extern crate futures;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate nom;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_process;
extern crate uuid;

mod dispatch;
mod error;
mod message;
mod state;
mod types;
mod util;

use clap::{App, Arg};
use dispatch::init_dispatch;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::prelude::*;

pub mod config {
    use std::sync::RwLock;
    pub struct Defaulted<T: Clone> {
        default: T,
        value: Option<T>,
    }

    impl<T: Clone> Defaulted<T> {
        fn new(default: T) -> Self {
            Defaulted {
                default,
                value: None,
            }
        }

        pub fn get(&self) -> T {
            match self.value {
                Some(ref value) => value.clone(),
                None => self.default.clone(),
            }
        }

        pub fn set(&mut self, value: T) {
            self.value = Some(value);
        }
    }

    pub struct Config {
        pub git_path: Defaulted<String>,
    }

    lazy_static! {
        pub static ref CONFIG: RwLock<Config> = RwLock::new(Config {
            git_path: Defaulted::new(String::from("git"))
        });
    }
}

pub fn main() {
    let matches = App::new("Git-RS")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Axosoft")
        .about("Run Git commands over a TCP interface")
        .arg(
            Arg::with_name("git-path")
                .short("g")
                .long("git-path")
                .value_name("GIT_PATH")
                .help("Sets the location of the Git binary. If it is not set, path is assumed.")
                .takes_value(true),
        )
        .get_matches();

    {
        let mut config = config::CONFIG.write().unwrap();
        config
            .git_path
            .set(String::from(matches.value_of("git-path").unwrap_or("git")));
    }

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
