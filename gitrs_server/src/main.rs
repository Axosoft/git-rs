extern crate bytes;
#[macro_use]
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

#[macro_export]
macro_rules! debug {
    ($block:block) => {
        if config::CONFIG.read().unwrap().debug {
            $block
        }
    };
}

mod config;
mod constants;
mod dispatch;
mod error;
mod message;
mod state;
mod types;
mod util;

use clap::{App, Arg};
use dispatch::init_dispatch;
use std::path::Path;
use std::process;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::prelude::*;

pub fn main() {
    let matches = App::new("Git-RS")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Axosoft")
        .about("Run Git commands over a TCP interface")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("The listen port of the server.")
                .validator(|maybe_port| match maybe_port.parse::<u32>() {
                    Ok(port) => if 1024 <= port && port <= 49151 {
                        Ok(())
                    } else {
                        Err(String::from("Must be a number between 1024 and 49151!"))
                    },
                    Err(_) => Err(String::from("Must be a number between 1024 and 49151!")),
                }),
        )
        .arg(
            Arg::with_name("git-path")
                .short("g")
                .long("git-path")
                .value_name("GIT_PATH")
                .help("Sets the location of the parent folder for Git binary. Will take precedence over existing system Git executable.")
                .takes_value(true)
                .validator(|maybe_path| {
                    if Path::new(&maybe_path).is_relative() {
                        Err(String::from("Must be an absolute path to parent folder of Git binary!"))
                    } else {
                        Ok(())
                    }
                }),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .hidden(true),
        )
        .get_matches();

    {
        let mut config = config::CONFIG.write().unwrap();
        matches.value_of("git-path").map(|maybe_path| {
            if Path::new(&maybe_path).is_dir() {
                config.git_path = Some(String::from(maybe_path));
            } else {
                process::exit(constants::exit_code::ENOENT);
            }
        });
        if matches.is_present("port") {
            // failure case should never happen because we have already validated the port.
            config.port = value_t!(matches.value_of("port"), u32).unwrap_or_else(|e| e.exit());
        }
        config.debug = matches.is_present("debug");
    }

    let state = Arc::new(Mutex::new(state::Shared::new()));
    let server_address = String::from(format!("0.0.0.0:{:?}", config::CONFIG.read().unwrap().port))
        .parse()
        .unwrap_or_else(|_| process::exit(constants::exit_code::EFAULT));

    let listener = TcpListener::bind(&server_address).unwrap_or_else(|_| {
        debug!({
            println!("TCP listener could not be bound to address!");
        });
        process::exit(constants::exit_code::EADDRINUSE);
    });

    let server = listener
        .incoming()
        .for_each(move |socket| {
            debug!({
                println!("accepted socket; addr={:?}", socket.peer_addr().unwrap());
            });
            init_dispatch(state.clone(), socket);
            Ok(())
        })
        .map_err(|err| {
            debug!({ eprintln!("accept error = {:?}", err) });
        });

    println!("{:?}", config::CONFIG.read().unwrap().port);
    tokio::run(server);
}
