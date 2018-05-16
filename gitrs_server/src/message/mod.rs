mod git_command;

pub mod protocol {
    pub use super::git_command::protocol as git_command;

    use semver::Version;
    use error::protocol::ErrorCode;

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type")]
    pub enum Inbound {
        Hello,
        GitCommand(git_command::Inbound),
        Goodbye,
    }

    #[derive(Debug, Serialize)]
    #[serde(tag = "type")]
    pub enum Outbound {
        Hello { version: Version },
        GladToMeetYou,
        Goodbye { error_code: Option<ErrorCode> },
    }
}

pub mod channel {
    pub enum Message {
        Noop,
    }
}
