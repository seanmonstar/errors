# errors

`std::error::Error` extensions

This crate encourages usage of the `std::error::Error` trait for
describing errors, providing the following utilities:

- **Error creation**: The `errors::new`, `errors::wrap`,
  and `errors::opaque` functions ease the creation of simple
  error values.
- **Error inspection**: Error source chains can be easily iterated with
  `errors::iter` iterators to find the error you're looking for.
- **Error formatting**: The error values created with this crate provide
  simple yet powerful control over the formatting of errors and their
  source chains, and the `errors::fmt` adapter allows
  foreign error values to follow along.

## Configuring Error Formatting

An `Error` likely has a message, it might have a cause, and someday, it may have a trace/frame. How should they be formatted? What is a good default, and how should a user configure to their needs?

### Output options:

- Top message only

  ```
  ship exploded
  ```
- Top message + message of source chain
  
  ```
  ship exploded: cat hair in generator
  ```
- Top message + message of source chain + trace/frame

  ```
  ship exploded
      at main.rs:55
      at ship.rs:89
  Caused by: cat hair in generator
      at ship::parts::generator.rs:33
      at ship::parts::engine.rs:789
      at ship.rs:89
      at main.rs:55
  ```

### Format flags

- **Default (`{}`)**: Print only the top-level message. This is inline with the recommendation for `Error`
  - *Example*: `println!("top only = {}", err)` outputs `top only = ship exploded`.
  - *Alternative*: This could also be achieved and possibly clearer by setting the "precision" flag to 0, such as `println!("top only: {:.0}", err)`.
- **Message chain (`{:+}`)**: Prints the message, and the message of each source, joined by `": "`.
  - *Example*: `println!("chain = {:+}", err)` outputs `chain = ship exploded: cat hair in generator`.
- **With trace/frame (`{:#}`)**: Prints the message and stack trace/frame
  - *Example*: `println!("top trace = {:#}", err)` outputs `top trace = ship exploded\n    at ship.rs:89`.
- **Message chain with trace/frame (`{:+#}`)**: Prints the message and stack trace/frame, and message and trace for each source, joined by `\nCaused by:`.
- **Message chain maximum (`{:+.2}`)**: Sets the maximum messages that should be printed down the source chain.
