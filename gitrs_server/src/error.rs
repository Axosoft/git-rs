pub mod protocol {
    use std::str;

    #[derive(Debug, Serialize)]
    #[serde(tag = "type", content = "message")]
    pub enum ErrorCode {
        BadRequest(String),
    }

    #[derive(Debug)]
    pub enum DeserializationError {
        Io,
        Syntax,
        Data,
        Encoding,
        Eof,
    }

    #[derive(Debug)]
    pub enum TcpSendError {
        Io,
    }

    #[derive(Debug)]
    pub enum TcpReceiveError {
        Io,
    }

    #[derive(Debug)]
    pub enum InboundMessageError {
        Unexpected,
    }

    #[derive(Debug)]
    pub enum ProcessError {
        Encoding,
        Failed,
    }

    #[derive(Debug)]
    pub enum Error {
        Deserialization(DeserializationError),
        InboundMessage(InboundMessageError),
        Process(ProcessError),
        TcpReceive(TcpReceiveError),
        TcpSend(TcpSendError),
    }

    impl From<str::Utf8Error> for Error {
        fn from(_error: str::Utf8Error) -> Self {
            Error::Deserialization(DeserializationError::Encoding)
        }
    }

    impl From<::serde_json::error::Error> for Error {
        fn from(error: ::serde_json::error::Error) -> Self {
            use serde_json::error::Category;

            Error::Deserialization(match error.classify() {
                Category::Io => DeserializationError::Io,
                Category::Syntax => DeserializationError::Syntax,
                Category::Data => DeserializationError::Data,
                Category::Eof => DeserializationError::Eof,
            })
        }
    }
}
