//! Binary protocol codecs and typed application messages.
//!
//! The crate exposes the protocol primitives under [`protocol`] and provides direct entry
//! points for both application-message directions:
//!
//! - [`to_client`] contains messages sent by a server to a client.
//! - [`to_server`] contains messages sent by a client to a server.
//!
//! The directional modules are also available under [`protocol::messages`].
//!
//! # Encoding a client message
//!
//! ```no_run
//! use luanti_protocol_rs::{to_server, protocol::messages::EncodeMessage};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let message = to_server::ChatMessage {
//!     message: String::from("Hello"),
//! };
//! let encoded = message.encode()?;
//! # let _ = encoded;
//! # Ok(())
//! # }
//! ```

pub mod errors;
pub mod protocol;

/// Application messages sent by a server to a client.
pub use protocol::messages::to_client;
/// Application messages sent by a client to a server.
pub use protocol::messages::to_server;

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub mod fuzz;

pub use errors::ProtocolError;

#[cfg(test)]
mod tests;
