//! `std::error::Error` extensions

use std::error::Error as StdError;

type Error = Box<dyn StdError + Send + Sync>;
type ErrorRef = dyn StdError + 'static;

pub mod fmt;
pub mod iter;
mod new;

pub use self::new::{opaque, wrap};

