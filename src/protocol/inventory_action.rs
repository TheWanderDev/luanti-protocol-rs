use bytes::Bytes;

use crate::errors::ProtocolError;
use crate::protocol::commands::ToServerCommand;
use crate::protocol::writer::PacketWriter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryLocation {
    CurrentPlayer,
    Player(String),
    NodeMeta { x: i16, y: i16, z: i16 },
    Detached(String),
}

impl InventoryLocation {
    fn serialize(&self) -> Result<String, ProtocolError> {
        match self {
            Self::CurrentPlayer => Ok("current_player".to_owned()),
            Self::Player(name) => {
                validate_token(name)?;
                Ok(format!("player:{name}"))
            }
            Self::NodeMeta { x, y, z } => Ok(format!("nodemeta:{x},{y},{z}")),
            Self::Detached(name) => {
                validate_token(name)?;
                Ok(format!("detached:{name}"))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryAction {
    Move {
        count: u16,
        from_inv: InventoryLocation,
        from_list: String,
        from_index: i16,
        to_inv: InventoryLocation,
        to_list: String,
        to_index: i16,
    },
    MoveSomewhere {
        count: u16,
        from_inv: InventoryLocation,
        from_list: String,
        from_index: i16,
        to_inv: InventoryLocation,
        to_list: String,
    },
    Drop {
        count: u16,
        from_inv: InventoryLocation,
        from_list: String,
        from_index: i16,
    },
    Craft {
        count: u16,
        inventory: InventoryLocation,
    },
}

fn validate_token(value: &str) -> Result<(), ProtocolError> {
    if value.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue("inventory action token"));
    }

    for byte in value.bytes() {
        if byte.is_ascii_whitespace() {
            return Err(ProtocolError::InvalidPlayerValue("inventory action token"));
        }
    }

    Ok(())
}

fn serialize_action(action: &InventoryAction) -> Result<String, ProtocolError> {
    let value = match action {
        InventoryAction::Move {
            count,
            from_inv,
            from_list,
            from_index,
            to_inv,
            to_list,
            to_index,
        } => {
            validate_token(from_list)?;
            validate_token(to_list)?;

            if *from_index < 0 || *to_index < 0 {
                return Err(ProtocolError::InvalidPlayerValue("inventory move"));
            }

            format!(
                "Move {count} {} {from_list} {from_index} {} {to_list} {to_index}",
                from_inv.serialize()?,
                to_inv.serialize()?
            )
        }
        InventoryAction::MoveSomewhere {
            count,
            from_inv,
            from_list,
            from_index,
            to_inv,
            to_list,
        } => {
            validate_token(from_list)?;
            validate_token(to_list)?;

            if *from_index < 0 {
                return Err(ProtocolError::InvalidPlayerValue("inventory move"));
            }

            format!(
                "MoveSomewhere {count} {} {from_list} {from_index} {} {to_list}",
                from_inv.serialize()?,
                to_inv.serialize()?
            )
        }
        InventoryAction::Drop {
            count,
            from_inv,
            from_list,
            from_index,
        } => {
            validate_token(from_list)?;

            if *from_index < 0 {
                return Err(ProtocolError::InvalidPlayerValue("inventory drop"));
            }

            format!(
                "Drop {count} {} {from_list} {from_index}",
                from_inv.serialize()?
            )
        }
        InventoryAction::Craft { count, inventory } => {
            format!("Craft {count} {} ", inventory.serialize()?)
        }
    };

    Ok(value)
}

pub fn encode_inventory_action(action: &InventoryAction) -> Result<Bytes, ProtocolError> {
    let serialized = serialize_action(action)?;

    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::InventoryAction));
    writer.write_bytes(serialized.as_bytes());

    Ok(writer.into_inner())
}
