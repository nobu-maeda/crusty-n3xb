use std::{error::Error, fmt};

pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub enum N3xbError {
    Simple(String),
    TagParsing(String),
    StrumParsing(strum::ParseError),
    CurrencyParsing(iso_currency::ParseCurrencyError),
    NostrClient(nostr_sdk::client::Error),
    NostrEvent(nostr_sdk::event::Error),
    SerdesJson(serde_json::Error),
    MpscSend(String),
}

impl Error for N3xbError {}

impl fmt::Display for N3xbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_string = match self {
            N3xbError::Simple(msg) => format!("n3xB-Error | Other - {}", msg),
            N3xbError::TagParsing(tag) => {
                format!("n3xB-Error | TagParsing - Cannot parse tag {}", tag)
            }
            N3xbError::StrumParsing(err) => {
                format!("n3xB-Error | StrumParseError - {}", err.to_string())
            }
            N3xbError::CurrencyParsing(err) => {
                format!("n3xB-Error | ParseCurrencyError - {}", err.to_string())
            }
            N3xbError::NostrClient(err) => {
                format!("n3xB-Error | NostrClientError - {}", err.to_string())
            }
            N3xbError::NostrEvent(err) => {
                format!("n3xB-Error | NostrEventError - {}", err.to_string())
            }
            N3xbError::SerdesJson(err) => {
                format!("n3xB-Error | SerdesJsonError - {}", err.to_string())
            }
            N3xbError::MpscSend(msg) => {
                format!("n3xB-Error | MpscSendError - {}", msg)
            }
        };
        write!(f, "{}", error_string)
    }
}

impl From<strum::ParseError> for N3xbError {
    fn from(e: strum::ParseError) -> N3xbError {
        N3xbError::StrumParsing(e)
    }
}

impl From<iso_currency::ParseCurrencyError> for N3xbError {
    fn from(e: iso_currency::ParseCurrencyError) -> N3xbError {
        N3xbError::CurrencyParsing(e)
    }
}

impl From<nostr_sdk::client::Error> for N3xbError {
    fn from(e: nostr_sdk::client::Error) -> N3xbError {
        N3xbError::NostrClient(e)
    }
}

impl From<nostr_sdk::event::Error> for N3xbError {
    fn from(e: nostr_sdk::event::Error) -> N3xbError {
        N3xbError::NostrEvent(e)
    }
}

impl From<serde_json::Error> for N3xbError {
    fn from(e: serde_json::Error) -> N3xbError {
        N3xbError::SerdesJson(e)
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for N3xbError {
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> N3xbError {
        N3xbError::MpscSend(e.to_string())
    }
}
