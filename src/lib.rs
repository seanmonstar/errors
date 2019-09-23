//! `std::error::Error` extensions

use std::error::Error;

type BoxError = Box<dyn Error + Send + Sync>;
type ErrorRef = dyn Error + 'static;

pub mod fmt;
pub mod iter;
mod new;

pub use self::new::{new, opaque, wrap};

