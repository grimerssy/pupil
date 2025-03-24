pub mod config;
pub mod http;

mod error;

use error::Error;

pub type Result<T> = std::result::Result<T, self::Error>;
