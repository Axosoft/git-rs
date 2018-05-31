pub mod protocol {
    #[derive(Debug, Deserialize)]
    pub enum Inbound {
        IsAncestor {
            ancestor_sha: String,
            descendant_sha: String,
        },
    }
}
