use std::{error::Error, fmt};

use iso_currency::ParseCurrencyError;

#[derive(Debug)]
pub enum N3xbError {
    Simple(String),
    TagParsing(String),
    StrumParsing(strum::ParseError),
    CurrencyParsing(ParseCurrencyError),
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
    fn from(e: ParseCurrencyError) -> N3xbError {
        N3xbError::CurrencyParsing(e)
    }
}
