mod echo;

pub mod protocol {
    pub use super::echo::protocol as echo;

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type")]
    pub enum Inbound {
        Echo(echo::Inbound),
    }
}
