//! `std::error::Error` extensions
//!
//! This crate encourages usage of the `std::error::Error` trait for
//! describing errors, providing the following utilities:
//!
//! - **Error creation**: The [`errors::new`][new], [`errors::wrap`][wrap],
//!   and [`errors::opaque`][opaque] functions ease the creation of simple
//!   error values.
//! - **Error inspection**: Error source chains can be easily iterated with
//!   [`errors::iter`][iter] iterators to find the error you're looking for.
//! - **Error formatting**: The error values created with this crate provide
//!   simple yet powerful control over the formatting of errors and their
//!   source chains, and the [`errors::fmt::chain`][fmt::chain] adapter allows
//!   foreign error values to follow along.
//!
//! # Creating Errors
//!
//! When an error condition has nothing special about besides a message, you
//! can create one easily with [`errors::new`][new]:
//!
//! ```
//! let err = errors::new("out of memory");
//! ```
//!
//! Library authors are encouraged to make distinct error types that may offer
//! domain-specific ways of describing and inspecting details of a certain
//! error case. For example:
//!
//! ```
//! use std::fmt;
//!
//! #[derive(Debug)]
//! struct TimedOut;
//!
//! impl fmt::Display for TimedOut {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         f.write_str("operation timed out")
//!     }
//! }
//!
//! impl std::error::Error for TimedOut {}
//! ```
//!
//! # Inspecting Errors
//!
//! # Formatting Errors

use std::error::Error;

type BoxError = Box<dyn Error + Send + Sync>;
type ErrorRef = dyn Error + 'static;

pub mod fmt;
pub mod iter;
mod new;

pub use self::fmt::Main;
pub use self::new::{new, opaque, wrap};

