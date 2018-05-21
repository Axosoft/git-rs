pub mod protocol {
    #[derive(Debug, Deserialize)]
    pub enum Inbound {
        Bisect { bad: String, good: String },
        Log,
        OpenRepo { path: String },
        Status,
    }
}
