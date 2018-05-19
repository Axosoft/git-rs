pub mod protocol {
    #[derive(Debug, Deserialize)]
    pub enum Inbound {
        Bisect { bad: String, good: String },
        OpenRepo { path: String },
        Status,
    }
}
