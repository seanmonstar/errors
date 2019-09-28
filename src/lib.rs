#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]

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
//! Errors tend to wrap others to provide more context. At times, we may wish
//! to programatically inspect the error and try to handle them depending on
//! what failed. We can do this by inspecting the source chain of an error with
//! the tools in [`errors::iter`][iter].
//!
//! Say we wanted to check for timeout errors and retry them:
//!
//! ```no_run
//! # use std::fmt;
//! #
//! # #[derive(Debug)]
//! # struct TimedOut;
//! #
//! # impl fmt::Display for TimedOut {
//! #     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//! #         f.write_str("operation timed out")
//! #     }
//! # }
//! #
//! # impl std::error::Error for TimedOut {}
//! # fn do_the_thing() -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
//! if let Err(e) = do_the_thing() {
//!     if errors::iter::chain(&*e).any(|err| err.is::<TimedOut>()) {
//!         do_the_thing(); // again!
//!     }
//! }
//! ```
//!
//! On the other hand, sometimes we want to wrap an error so that it can help
//! users debug the problem, but we *don't* want them to programmatically react
//! to the error.
//!
//! Say after trying to `do_the_thing` repeated after 3 timeouts, we wanted to
//! include that information, but prevent the user from thinking the error is
//! still a `Timeout`?
//!
//! This can be easily done with [`errors::opaque`](opaque):
//!
//! ```no_run
//! # fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//! # use std::fmt;
//! #
//! # #[derive(Debug)]
//! # struct TimedOut;
//! #
//! # impl fmt::Display for TimedOut {
//! #     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//! #         f.write_str("operation timed out")
//! #     }
//! # }
//! #
//! # impl std::error::Error for TimedOut {}
//! # fn do_the_thing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> { Ok(()) }
//! let mut cnt = 0;
//! loop {
//!     if let Err(e) = do_the_thing() {
//!         if errors::is::<TimedOut>(&*e) {
//!             if cnt < 3 {
//!                 cnt += 1;
//!                 continue; // again!
//!             }
//!
//!             // stop retrying
//!             return Err(errors::wrap("too many attempts", errors::opaque(e)));
//!
//!         }
//!
//!         // something else went boom
//!         return Err(e);
//!     }
//!     return Ok(()); // success!
//! }
//! # }
//! ```
//!
//! # Formatting Errors
//!
//! This crate defines a way for a user to specify how to easily format an
//! error along with its source chain. All the error values created with this
//! crate follow this spec, and any other errors can be adapted with the handy
//! [`errors::fmt`](fmt) adapter.
//!
//! ### Output options:
//!
//! - Top message only
//!
//!   ```plain
//!   ship exploded
//!   ```
//! - Top message + message of source chain
//!
//!   ```plain
//!   ship exploded: cat hair in generator
//!   ```
//! - Top message + message of source chain + trace/frame
//!
//!   ```plain
//!   ship exploded
//!       at main.rs:55
//!       at ship.rs:89
//!   Caused by: cat hair in generator
//!       at ship::parts::generator.rs:33
//!       at ship::parts::engine.rs:789
//!       at ship.rs:89
//!       at main.rs:55
//!   ```
//!
//! ### Format Flags
//!
//! - **Default (`{}`)**: Print only the top-level message. This is inline with the recommendation for `Error`
//!   - *Example*: `println!("top only = {}", err)` outputs `top only = ship exploded`.
//!   - *Alternative*: This could also be achieved and possibly clearer by setting the "precision" flag to 0, such as `println!("top only: {:.0}", err)`.
//! - **Message chain (`{:+}`)**: Prints the message, and the message of each source, joined by `": "`.
//!   - *Example*: `println!("chain = {:+}", err)` outputs `chain = ship exploded: cat hair in generator`.
//! - **With trace/frame (`{:#}`)**: Prints the message and stack trace/frame
//!   - *Example*: `println!("top trace = {:#}", err)` outputs `top trace = ship exploded\n    at ship.rs:89`.
//! - **Message chain with trace/frame (`{:+#}`)**: Prints the message and stack trace/frame, and message and trace for each source, joined by `\nCaused by:`.
//!
//!
//! ## `errors::Main`
//!
//! Newer versions of Rust allow returning a `Result` from the `main` function
//! and it will be formatted and printed to the user. Using the
//! [`errors::Main`](Main) type, you can easily convert any application errors
//! such that the full source chain will be printed in a useful format.
//!
//! ```
//! # mod not_main {
//! # use std::fmt;
//! #
//! # #[derive(Debug)]
//! # struct TimedOut;
//! #
//! # impl fmt::Display for TimedOut {
//! #     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//! #         f.write_str("operation timed out")
//! #     }
//! # }
//! #
//! # impl std::error::Error for TimedOut {}
//! fn main() -> Result<(), errors::Main> {
//!     do_the_first_thing()?;
//!     do_the_second()?;
//!     Ok(())
//! }
//!
//! // These even have different error types!
//!
//! fn do_the_first_thing() -> Result<(), TimedOut> {
//!     // ...
//!     # Ok(())
//! }
//!
//! fn do_the_second() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!     // ...
//!     # Ok(())
//! }
//! # }
//! ```

use std::error::Error;

type BoxError = Box<dyn Error + Send + Sync>;
type ErrorRef = dyn Error + 'static;

mod fmt;
pub mod iter;
mod new;

pub use self::fmt::{fmt, Main};
pub use self::iter::{find, is};
pub use self::new::{new, opaque, wrap};

