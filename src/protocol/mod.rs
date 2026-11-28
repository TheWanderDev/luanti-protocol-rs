//! Protocol primitives, wire models, and typed application-message codecs.
//!
//! Prefer [`messages::to_client`] and [`messages::to_server`] for complete application messages.
//! Their codec traits validate command identifiers and make version-dependent decoding explicit.
//! Use the lower-level modules when a caller needs a standalone payload codec, MTP datagram
//! framing, or access to decoded domain models.
//!
//! # Layer boundaries
//!
//! - [`packet`] handles one MTP datagram at a time. It does not reassemble split messages.
//! - [`reader`] and [`writer`] handle primitive fields in wire order.
//! - [`messages`] handles complete application messages and command identifiers.
//! - Other modules provide payload models and standalone codecs for world, inventory, media,
//!   entities, and server state.
//!
//! Sequence management, retransmission, ACK processing, split assembly, sockets, and session
//! lifecycle are outside this module.

pub mod active_object;
pub mod chat;
pub mod client_commands;
pub mod commands;
pub mod compression;
pub mod constant;
pub mod formspec;
pub mod hud;
pub mod inventory;
pub mod inventory_action;
pub mod itemdef;
pub mod mapblock;
pub mod media;
pub mod messages;
pub mod minimap;
pub mod mod_channel;
pub mod node_metadata;
pub mod node_transform;
pub mod nodedef;
pub mod packet;
pub mod particle;
pub mod player;
pub mod reader;
pub mod server_state;
pub mod status;
pub mod tiledef;
pub mod vectors;
pub mod writer;
