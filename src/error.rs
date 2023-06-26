use std::{error::Error, fmt};

#[derive(Debug)]
pub enum N3xbError {
  Other(String),
}

impl Error for N3xbError {}

impl fmt::Display for N3xbError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let error_string = 
    match self {
      N3xbError::Other(msg) => format!("n3xB-Error | Other - {}", msg),
    };
    write!(f, "{}", error_string)
  }
}