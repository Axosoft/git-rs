mod merge_base;

pub mod protocol {
    pub use super::merge_base::protocol as merge_base;

    #[derive(Debug, Deserialize)]
    pub enum Inbound {
        Bisect { bad: String, good: String },
        Log,
        MergeBase(merge_base::Inbound),
        OpenRepo { path: String },
        Status,
    }
}
