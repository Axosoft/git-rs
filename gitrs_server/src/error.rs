pub mod protocol {
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
    pub enum Error {
        Deserialization(DeserializationError),
        InboundMessage(InboundMessageError),
        TcpReceive(TcpReceiveError),
        TcpSend(TcpSendError),
    }

    pub mod serde_json {
        use super::{DeserializationError, Error};

        pub fn to_error(error: ::serde_json::error::Error) -> Error {
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
