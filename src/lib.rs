pub mod config;
pub mod http;
pub mod telemetry;

mod error;

use error::Error;

pub type Result<T> = std::result::Result<T, self::Error>;
