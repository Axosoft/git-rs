mod echo;

pub mod protocol {
    pub use super::echo::protocol as echo;

    #[derive(Debug, Deserialize)]
    pub enum Inbound {
        Echo(echo::Inbound),
        OpenRepo { path: String },
    }
}
