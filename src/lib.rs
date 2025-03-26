pub mod config;
pub mod telemetry;
pub mod http;

mod error;

use error::Error;

pub type Result<T> = std::result::Result<T, self::Error>;
