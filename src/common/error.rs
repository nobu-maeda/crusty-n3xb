use std::{error::Error, fmt, io};

use serde::{Deserialize, Serialize};
use strum_macros::{Display, IntoStaticStr};

pub type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub enum N3xbError {
    Simple(String),
    InvalidOffer(OfferInvalidReason),
    TagParsing(String),
    StrumParsing(strum::ParseError),
    CurrencyParsing(iso_currency::ParseCurrencyError),
    NostrClient(nostr_sdk::client::Error),
    NostrEvent(nostr_sdk::event::Error),
    SerdesJson(serde_json::Error),
    MpscSend(String),
    Io(io::Error),
    JoinError(tokio::task::JoinError),
    OneshotRecv(tokio::sync::oneshot::error::RecvError),
}

impl Error for N3xbError {}

impl fmt::Display for N3xbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_string = match self {
            N3xbError::Simple(msg) => format!("n3xB-Error | Simple - {}", msg),
            N3xbError::InvalidOffer(reason) => {
                format!("n3xB-Error | InvalidOffer - {}", reason)
            }
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
            N3xbError::MpscSend(err) => {
                format!("n3xB-Error | MpscSendError - {}", err.to_string())
            }
            N3xbError::Io(err) => format!("n3xB-Error | IoError - {}", err.to_string()),
            N3xbError::JoinError(err) => {
                format!("n3xB-Error | JoinError - {}", err.to_string())
            }
            N3xbError::OneshotRecv(err) => {
                format!("n3xB-Error | RecvError - {}", err.to_string())
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

impl From<OfferInvalidReason> for N3xbError {
    fn from(e: OfferInvalidReason) -> N3xbError {
        N3xbError::InvalidOffer(e)
    }
}

impl From<io::Error> for N3xbError {
    fn from(e: io::Error) -> N3xbError {
        N3xbError::Io(e)
    }
}

impl From<tokio::task::JoinError> for N3xbError {
    fn from(e: tokio::task::JoinError) -> N3xbError {
        N3xbError::JoinError(e)
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for N3xbError {
    fn from(e: tokio::sync::oneshot::error::RecvError) -> N3xbError {
        N3xbError::OneshotRecv(e)
    }
}

#[derive(Clone, Display, IntoStaticStr, PartialEq, Serialize, Deserialize)]
pub enum OfferInvalidReason {
    Cancelled,
    PendingAnother,
    DuplicateOffer,
    TransactedSatAmountFractional,
    MakerObligationKindInvalid,
    MakerObligationAmountInvalid,
    MakerBondInvalid,
    TakerObligationKindInvalid,
    TakerObligationAmountInvalid,
    TakerBondInvalid,
    ExchangeRateInvalid,
    MarketOracleInvalid,
    TradeEngineSpecific,
    PowTooHigh,
}

impl fmt::Debug for OfferInvalidReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OfferInvalidReason::Cancelled => write!(f, "Order have been cancelled"),
            OfferInvalidReason::PendingAnother => write!(f, "Order is pending another Taker"),
            OfferInvalidReason::DuplicateOffer => write!(f, "Offer already previously received"),
            OfferInvalidReason::MakerObligationKindInvalid => write!(
                f,
                " Maker obligation kind is invalid or not in acceptable set"
            ),
            OfferInvalidReason::TransactedSatAmountFractional => {
                write!(f, "Transacted sat amount cannot be fractional")
            }
            OfferInvalidReason::MakerObligationAmountInvalid => write!(
                f,
                "Maker obligation amount is invalid or not in acceptable range"
            ),
            OfferInvalidReason::MakerBondInvalid => {
                write!(f, "Maker bond is invalid or not in acceptable range")
            }
            OfferInvalidReason::TakerObligationKindInvalid => write!(
                f,
                "Taker obligation kind is invalid or not in acceptable set"
            ),
            OfferInvalidReason::TakerObligationAmountInvalid => write!(
                f,
                "Taker obligation amount is invalid or not in acceptable range"
            ),
            OfferInvalidReason::TakerBondInvalid => {
                write!(f, "Taker bond is invalid or not in acceptable range")
            }
            OfferInvalidReason::ExchangeRateInvalid => {
                write!(f, "Exchange rate specified is invalid")
            }
            OfferInvalidReason::MarketOracleInvalid => write!(
                f,
                "Market oracle specified is invalid or not in acceptable set"
            ),
            OfferInvalidReason::TradeEngineSpecific => {
                write!(f, "Reason provided in trade_engine_specifics JSON")
            }
            OfferInvalidReason::PowTooHigh => {
                write!(f, "The Taker desired minimum PoW is too high for the Maker")
            }
        }
    }
}
