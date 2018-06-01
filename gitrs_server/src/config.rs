use std::sync::RwLock;

pub struct Config {
    pub debug: bool,
    pub git_path: Option<String>,
    pub exec_path: Option<String>,
    pub port: u32,
}

lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config {
        debug: false,
        git_path: None,
        exec_path: None,
        port: 5134,
    });
}
