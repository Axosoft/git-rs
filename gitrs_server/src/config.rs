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
    pub debug: bool,
    pub git_path: Defaulted<String>,
    pub port: u32,
}

lazy_static! {
    pub static ref CONFIG: RwLock<Config> = RwLock::new(Config {
        debug: false,
        git_path: Defaulted::new(String::from("git")),
        port: 5134,
    });
}
