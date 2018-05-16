pub mod protocol {
    #[derive(Debug, Deserialize)]
    pub struct Inbound {
        input: String,
    }
}
