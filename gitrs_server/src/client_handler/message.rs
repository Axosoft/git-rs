pub mod protocol {
    use semver::Version;

    #[derive(Serialize)]
    #[serde(tag = "type", content = "message")]
    pub enum ErrorCode {
        Ok,
        BadRequest(String),
    }

    #[derive(Deserialize)]
    #[serde(tag = "type")]
    pub enum InboundMessage {
        Hello,
        Goodbye,
    }

    #[derive(Serialize)]
    #[serde(tag = "type")]
    pub enum OutboundMessage {
        Hello { version: Version },
        Goodbye { error_code: Option<ErrorCode> },
    }
}

pub mod channel {
    pub enum Message {
        Noop,
    }
}
