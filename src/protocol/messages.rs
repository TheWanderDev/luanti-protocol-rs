use crate::errors::ProtocolError;
use crate::protocol::commands::{ToClientCommand, ToServerCommand};
use crate::protocol::constant::{
    CLIENT_VERSION_MAJOR, CLIENT_VERSION_MINOR, CLIENT_VERSION_PATCH, CLIENT_VERSION_STRING,
    FORMSPEC_API_VERSION, MAX_PROTOCOL_VERSION, MIN_PROTOCOL_VERSION, SERIALIZATION_VERSION,
};
use crate::protocol::reader::PacketReader;
use crate::protocol::writer::PacketWriter;
use bytes::Bytes;

#[derive(Debug, PartialEq)]
pub enum AuthServerMessage {
    Accepted(AuthAcceptData),
    SudoAccepted {
        auth_mechanisms: u32,
    },
    SudoDenied,
    Rejected {
        code: u8,
        message: String,
        reconnect: bool,
    },
    LegacyRejected {
        message: String,
    },
    SrpChallenge {
        salt: Vec<u8>,
        server_b: Vec<u8>,
    },
}

pub fn encode_init2(language: &str) -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::Init2));
    writer.write_string16(language)?;

    Ok(writer.into_inner())
}

pub fn encode_first_srp(
    salt: &[u8],
    verifier: &[u8],
    password_is_empty: bool,
) -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::FirstSrp));
    writer.write_binary_string16(salt)?;
    writer.write_binary_string16(verifier)?;
    writer.write_u8(u8::from(password_is_empty));

    Ok(writer.into_inner())
}

pub fn encode_srp_bytes_a(public_a: &[u8], legacy_password: bool) -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::SrpBytesA));
    writer.write_binary_string16(public_a)?;
    writer.write_u8(u8::from(!legacy_password));

    Ok(writer.into_inner())
}

pub fn encode_srp_bytes_m(proof: &[u8]) -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::SrpBytesM));
    writer.write_binary_string16(proof)?;

    Ok(writer.into_inner())
}

pub fn decode_access_denied_payload(data: &[u8]) -> Result<(u8, String, bool), ProtocolError> {
    let mut reader = PacketReader::new(data);
    decode_access_denied(&mut reader)
}

fn decode_access_denied(
    reader: &mut PacketReader<'_>,
) -> Result<(u8, String, bool), ProtocolError> {
    let code = reader.read_u8()?;
    let message = if reader.is_empty() {
        String::new()
    } else {
        String::from_utf8_lossy(reader.read_string16()?).into_owned()
    };
    let reconnect = if reader.is_empty() {
        false
    } else {
        reader.read_u8()? & 1 != 0
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidAuthPacket);
    }

    Ok((code, message, code == 6 || reconnect))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AuthAcceptData {
    pub map_seed: u64,
    pub recommended_send_interval: f32,
    pub sudo_auth_mechanisms: u32,
}

pub fn decode_auth_accept_payload(data: &[u8]) -> Result<AuthAcceptData, ProtocolError> {
    let mut reader = PacketReader::new(data);
    if reader.remaining() != 28 {
        return Err(ProtocolError::InvalidAuthPacket);
    }

    let _ = reader.read_f32()?;
    let _ = reader.read_f32()?;
    let _ = reader.read_f32()?;

    let map_seed = reader.read_u64()?;
    let recommended_send_interval = reader.read_f32()?;
    let sudo_auth_mechanisms = reader.read_u32()?;

    if !recommended_send_interval.is_finite() || recommended_send_interval <= 0.0 {
        return Err(ProtocolError::InvalidAuthPacket);
    }

    Ok(AuthAcceptData {
        map_seed,
        recommended_send_interval,
        sudo_auth_mechanisms,
    })
}

pub fn decode_sudo_mode_payload(data: &[u8]) -> Result<u32, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let auth_mechanisms = reader.read_u32()?;
    if !reader.is_empty() {
        return Err(ProtocolError::InvalidAuthPacket);
    }
    Ok(auth_mechanisms)
}

pub fn decode_sudo_mode_denial(data: &[u8]) -> Result<(), ProtocolError> {
    if data.is_empty() {
        Ok(())
    } else {
        Err(ProtocolError::InvalidAuthPacket)
    }
}

pub fn decode_auth_server_message(data: &[u8]) -> Result<AuthServerMessage, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let raw_command = reader.read_u16()?;

    let command = ToClientCommand::from_repr(raw_command)
        .ok_or(ProtocolError::UnexpectedCommand(raw_command))?;

    match command {
        ToClientCommand::AuthAccept => {
            let payload_length = reader.remaining();
            let payload = reader.read_bytes(payload_length)?;
            let data = decode_auth_accept_payload(payload)?;
            Ok(AuthServerMessage::Accepted(data))
        }

        ToClientCommand::AcceptSudoMode => {
            let payload_length = reader.remaining();
            let payload = reader.read_bytes(payload_length)?;
            Ok(AuthServerMessage::SudoAccepted {
                auth_mechanisms: decode_sudo_mode_payload(payload)?,
            })
        }

        ToClientCommand::DenySudoMode => {
            let payload_length = reader.remaining();
            let payload = reader.read_bytes(payload_length)?;
            decode_sudo_mode_denial(payload)?;
            Ok(AuthServerMessage::SudoDenied)
        }

        ToClientCommand::AccessDenied => {
            let (code, message, reconnect) = decode_access_denied(&mut reader)?;

            Ok(AuthServerMessage::Rejected {
                code,
                message,
                reconnect,
            })
        }

        ToClientCommand::AccessDeniedLegacy => {
            let message = reader.read_wide_string16()?;

            if !reader.is_empty() {
                return Err(ProtocolError::InvalidAuthPacket);
            }

            Ok(AuthServerMessage::LegacyRejected { message })
        }

        ToClientCommand::SrpBytesSB => {
            let salt = reader.read_string16()?.to_vec();
            let server_b = reader.read_string16()?.to_vec();

            if !reader.is_empty() {
                return Err(ProtocolError::InvalidAuthPacket);
            }

            Ok(AuthServerMessage::SrpChallenge { salt, server_b })
        }

        _ => Err(ProtocolError::UnexpectedCommand(raw_command)),
    }
}

pub fn encode_client_ready() -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::new();

    writer.write_u16(u16::from(ToServerCommand::ClientReady));
    writer.write_u8(CLIENT_VERSION_MAJOR);
    writer.write_u8(CLIENT_VERSION_MINOR);
    writer.write_u8(CLIENT_VERSION_PATCH);
    writer.write_u8(0);
    writer.write_string16(CLIENT_VERSION_STRING)?;
    writer.write_u16(FORMSPEC_API_VERSION);

    Ok(writer.into_inner())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitMessage {
    pub player_name: String,
    pub protocol_min: u16,
    pub protocol_max: u16,
    pub serialization_version: u8,
}

impl InitMessage {
    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        let mut reader = PacketReader::new(data);
        let raw_command = reader.read_u16()?;

        if raw_command != u16::from(ToServerCommand::Init) {
            return Err(ProtocolError::UnexpectedCommand(raw_command));
        }

        let serialization_version = reader.read_u8()?;
        let _compression_modes = reader.read_u16()?;
        let protocol_min = reader.read_u16()?;
        let protocol_max = reader.read_u16()?;
        let player_name = std::str::from_utf8(reader.read_string16()?)
            .map_err(|_| ProtocolError::InvalidHello)?
            .to_owned();

        if !reader.is_empty() || protocol_min > protocol_max || player_name.is_empty() {
            return Err(ProtocolError::InvalidHello);
        }
      
        if serialization_version < 27 {
            return Err(ProtocolError::InvalidSerializationVersion(
                serialization_version,
            ));
        }

        Ok(Self {
            player_name,
            protocol_min,
            protocol_max,
            serialization_version,
        })
    }

    pub fn encode(&self) -> Result<Bytes, ProtocolError> {
        if !(27..=SERIALIZATION_VERSION).contains(&self.serialization_version) {
            return Err(ProtocolError::InvalidSerializationVersion(
                self.serialization_version,
            ));
        }

        if !(MIN_PROTOCOL_VERSION..=MAX_PROTOCOL_VERSION).contains(&self.protocol_min)
            || !(MIN_PROTOCOL_VERSION..=MAX_PROTOCOL_VERSION).contains(&self.protocol_max)
            || self.protocol_min > self.protocol_max
        {
            return Err(ProtocolError::InvalidProtocolVersion(self.protocol_max));
        }

        let mut writer = PacketWriter::new();

        writer.write_u16(u16::from(ToServerCommand::Init));
        writer.write_u8(self.serialization_version);
        writer.write_u16(0);
        writer.write_u16(self.protocol_min);
        writer.write_u16(self.protocol_max);
        writer.write_string16(&self.player_name)?;

        Ok(writer.into_inner())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloMessage {
    pub serialization_version: u8,
    pub protocol_version: u16,
    pub auth_mechanisms: u32,
}

impl HelloMessage {
    pub fn encode(&self) -> Result<Bytes, ProtocolError> {
        if !(27..=SERIALIZATION_VERSION).contains(&self.serialization_version) {
            return Err(ProtocolError::InvalidSerializationVersion(
                self.serialization_version,
            ));
        }

        if !(MIN_PROTOCOL_VERSION..=MAX_PROTOCOL_VERSION).contains(&self.protocol_version) {
            return Err(ProtocolError::InvalidProtocolVersion(self.protocol_version));
        }

        let mut writer = PacketWriter::new();
        writer.write_u16(u16::from(ToClientCommand::Hello));
        writer.write_u8(self.serialization_version);
        writer.write_u16(0);
        writer.write_u16(self.protocol_version);
        writer.write_u32(self.auth_mechanisms);
        writer.write_string16("")?;

        Ok(writer.into_inner())
    }

    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        let mut reader = PacketReader::new(data);

        let raw_command = reader.read_u16()?;

        let command = ToClientCommand::from_repr(raw_command)
            .ok_or(ProtocolError::UnexpectedCommand(raw_command))?;

        if command != ToClientCommand::Hello {
            return Err(ProtocolError::UnexpectedCommand(raw_command));
        }

        let serialization_version = reader.read_u8()?;
        let _compression_modes = reader.read_u16()?;
        let protocol_version = reader.read_u16()?;
        let auth_mechanisms = reader.read_u32()?;

        reader.read_string16()?;

        if !reader.is_empty() {
            return Err(ProtocolError::InvalidHello);
        }

        if !(27..=SERIALIZATION_VERSION).contains(&serialization_version) {
            return Err(ProtocolError::InvalidSerializationVersion(
                serialization_version,
            ));
        }

        if !(MIN_PROTOCOL_VERSION..=MAX_PROTOCOL_VERSION).contains(&protocol_version) {
            return Err(ProtocolError::InvalidProtocolVersion(protocol_version));
        }

        Ok(Self {
            serialization_version,
            protocol_version,
            auth_mechanisms,
        })
    }
}
