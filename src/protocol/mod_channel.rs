use crate::errors::ProtocolError;
use crate::protocol::commands::ToServerCommand;
use crate::protocol::reader::PacketReader;
use crate::protocol::writer::PacketWriter;
use bytes::Bytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModChannelMessage {
    pub channel: String,
    pub sender: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModChannelAccess {
    Initializing,
    ReadWrite,
    ReadOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModChannelSignalKind {
    JoinOk,
    JoinFailure,
    LeaveOk,
    LeaveFailure,
    ChannelNotRegistered,
    SetState(ModChannelAccess),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModChannelSignalUpdate {
    pub channel: String,
    pub signal: ModChannelSignalKind,
}

pub(crate) fn decode_mod_channel_message(data: &[u8]) -> Result<ModChannelMessage, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let channel = decode_string(reader.read_string16()?);
    let sender = decode_string(reader.read_string16()?);
    let message = decode_string(reader.read_string16()?);

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidModChannel(
            "message has trailing bytes",
        ));
    }

    Ok(ModChannelMessage {
        channel,
        sender,
        message,
    })
}

pub(crate) fn decode_mod_channel_signal(
    data: &[u8],
) -> Result<Option<ModChannelSignalUpdate>, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let signal = reader.read_u8()?;
    let channel = decode_string(reader.read_string16()?);

    let signal = match signal {
        0 => ModChannelSignalKind::JoinOk,
        1 => ModChannelSignalKind::JoinFailure,
        2 => ModChannelSignalKind::LeaveOk,
        3 => ModChannelSignalKind::LeaveFailure,
        4 => ModChannelSignalKind::ChannelNotRegistered,
        5 => {
            let state = reader.read_u8()?;
            let access = match state {
                0 => ModChannelAccess::Initializing,
                1 => ModChannelAccess::ReadWrite,
                2 => ModChannelAccess::ReadOnly,
                _ => return Ok(None),
            };
            ModChannelSignalKind::SetState(access)
        }
        _ => return Ok(None),
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidModChannel(
            "signal has trailing bytes",
        ));
    }

    Ok(Some(ModChannelSignalUpdate { channel, signal }))
}

fn decode_string(value: &[u8]) -> String {
    String::from_utf8_lossy(value).into_owned()
}

pub(crate) fn encode_mod_channel_join(channel: &str) -> Result<Bytes, ProtocolError> {
    encode_mod_channel_name(ToServerCommand::ModChannelJoin, channel)
}

pub(crate) fn encode_mod_channel_leave(channel: &str) -> Result<Bytes, ProtocolError> {
    encode_mod_channel_name(ToServerCommand::ModChannelLeave, channel)
}

fn encode_mod_channel_name(
    command: ToServerCommand,
    channel: &str,
) -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::with_capacity(4 + channel.len());
    writer.write_u16(u16::from(command));
    writer.write_string16(channel)?;
    Ok(writer.into_inner())
}

pub(crate) fn encode_mod_channel_message(
    channel: &str,
    message: &str,
) -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::with_capacity(8 + channel.len() + message.len());
    writer.write_u16(u16::from(ToServerCommand::ModChannelMessage));
    writer.write_string16(channel)?;
    writer.write_string16(message)?;
    Ok(writer.into_inner())
}
