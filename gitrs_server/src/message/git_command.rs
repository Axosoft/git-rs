pub mod protocol {
    #[derive(Debug, Deserialize)]
    pub enum Inbound {
        OpenRepo { path: String },
        Status,
    }
}
