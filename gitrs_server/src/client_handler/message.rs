pub mod protocol {
    use semver::Version;

    #[derive(Debug, Serialize)]
    #[serde(tag = "type", content = "message")]
    pub enum ErrorCode {
        Ok,
        BadRequest(String),
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type")]
    pub enum InboundMessage {
        Hello,
        Goodbye,
    }

    #[derive(Debug, Serialize)]
    #[serde(tag = "type")]
    pub enum OutboundMessage {
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
