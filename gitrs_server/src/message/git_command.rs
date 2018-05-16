pub mod protocol {
    #[derive(Debug, Deserialize)]
    pub enum Inbound {
        Echo { input: String },
        OpenRepo { path: String },
        Status,
    }
}
