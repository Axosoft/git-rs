pub mod protocol {
    #[derive(Debug, Deserialize)]
    pub struct Inbound {
        pub input: String,
    }
}
