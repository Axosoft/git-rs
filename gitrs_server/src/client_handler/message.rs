pub mod protocol {
    use bytes::Bytes;
    use error::protocol::Error;
    use semver::Version;

    #[derive(Debug, Serialize)]
    #[serde(tag = "type", content = "message")]
    pub enum ErrorCode {
        BadRequest(String),
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type")]
    pub enum InboundMessage {
        Hello,
        Goodbye,
        RunGitCommand,
    }

    #[derive(Debug, Serialize)]
    #[serde(tag = "type")]
    pub enum OutboundMessage {
        Hello { version: Version },
        GladToMeetYou,
        Goodbye { error_code: Option<ErrorCode> },
    }

    pub fn deserialize(bytes: &::bytes::BytesMut) -> Result<InboundMessage, Error> {
        use std::str::from_utf8;

        from_utf8(&bytes)
            .map_err(Error::from)
            .and_then(|message| ::serde_json::from_str(&message).map_err(Error::from))
    }

    pub fn serialize(message: &OutboundMessage) -> Result<Bytes, Error> {
        let message = ::serde_json::to_string(message).map_err(Error::from)?;
        Ok(Bytes::from(message.into_bytes()))
    }
}

pub mod channel {
    pub enum Message {
        Noop,
    }
}
