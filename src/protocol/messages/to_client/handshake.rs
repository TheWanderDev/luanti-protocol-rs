use crate::errors::ProtocolError;
use crate::protocol::commands::{
    TOCLIENT_ACCEPT_SUDO_MODE, TOCLIENT_ACCESS_DENIED, TOCLIENT_ACCESS_DENIED_LEGACY,
    TOCLIENT_AUTH_ACCEPT, TOCLIENT_DENY_SUDO_MODE, TOCLIENT_HELLO, TOCLIENT_SRP_BYTES_SB,
};
use crate::protocol::constant::{
    MAX_PROTOCOL_VERSION, MIN_PROTOCOL_VERSION, SERIALIZATION_VERSION,
};
use crate::protocol::messages::{DecodeMessage, EncodeMessage, Message};
use crate::protocol::reader::PacketReader;
use crate::protocol::writer::PacketWriter;

/// Server handshake response to [`crate::protocol::messages::to_server::Init`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    pub serialization_version: u8,
    pub protocol_version: u16,
    pub auth_mechanisms: u32,
}

impl Message for Hello {
    const COMMAND_ID: u16 = TOCLIENT_HELLO;
}

impl EncodeMessage for Hello {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        if !(27..=SERIALIZATION_VERSION).contains(&self.serialization_version) {
            return Err(ProtocolError::InvalidSerializationVersion(
                self.serialization_version,
            ));
        }

        if !(MIN_PROTOCOL_VERSION..=MAX_PROTOCOL_VERSION).contains(&self.protocol_version) {
            return Err(ProtocolError::InvalidProtocolVersion(self.protocol_version));
        }

        let mut writer = PacketWriter::new();
        writer.write_u16(<Self as crate::protocol::messages::Message>::COMMAND_ID);
        writer.write_u8(self.serialization_version);
        writer.write_u16(0);
        writer.write_u16(self.protocol_version);
        writer.write_u32(self.auth_mechanisms);
        writer.write_string16("")?;
        Ok(writer.into_inner())
    }
}

impl DecodeMessage for Hello {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);

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

/// Authentication acceptance data sent by the server.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AuthAccept {
    pub map_seed: u64,
    pub recommended_send_interval: f32,
    pub sudo_auth_mechanisms: u32,
}

impl Message for AuthAccept {
    const COMMAND_ID: u16 = TOCLIENT_AUTH_ACCEPT;
}

impl DecodeMessage for AuthAccept {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);

        if reader.remaining() != 28 {
            return Err(ProtocolError::InvalidAuthPacket);
        }

        let _unused_position_x = reader.read_f32()?;
        let _unused_position_y = reader.read_f32()?;
        let _unused_position_z = reader.read_f32()?;
        let map_seed = reader.read_u64()?;
        let recommended_send_interval = reader.read_f32()?;
        let sudo_auth_mechanisms = reader.read_u32()?;

        if !recommended_send_interval.is_finite() || recommended_send_interval <= 0.0 {
            return Err(ProtocolError::InvalidAuthPacket);
        }

        Ok(Self {
            map_seed,
            recommended_send_interval,
            sudo_auth_mechanisms,
        })
    }
}

impl EncodeMessage for AuthAccept {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        if !self.recommended_send_interval.is_finite() || self.recommended_send_interval <= 0.0 {
            return Err(ProtocolError::InvalidAuthPacket);
        }
        let mut writer = PacketWriter::with_capacity(30);
        writer.write_u16(<Self as Message>::COMMAND_ID);
        writer.write_f32(0.0);
        writer.write_f32(0.0);
        writer.write_f32(0.0);
        writer.write_u64(self.map_seed);
        writer.write_f32(self.recommended_send_interval);
        writer.write_u32(self.sudo_auth_mechanisms);
        Ok(writer.into_inner())
    }
}

/// Confirms that the client entered sudo mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptSudoMode {
    pub auth_mechanisms: u32,
}

impl Message for AcceptSudoMode {
    const COMMAND_ID: u16 = TOCLIENT_ACCEPT_SUDO_MODE;
}

impl DecodeMessage for AcceptSudoMode {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);
        let auth_mechanisms = reader.read_u32()?;
        if !reader.is_empty() {
            return Err(ProtocolError::InvalidAuthPacket);
        }
        Ok(Self { auth_mechanisms })
    }
}

/// Reports that sudo-mode authentication was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DenySudoMode;

impl Message for DenySudoMode {
    const COMMAND_ID: u16 = TOCLIENT_DENY_SUDO_MODE;
}

impl DecodeMessage for DenySudoMode {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let reader = PacketReader::new(payload);
        if !reader.is_empty() {
            return Err(ProtocolError::InvalidAuthPacket);
        }
        Ok(Self)
    }
}

/// Reports an authentication failure using the current access-denied format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessDenied {
    pub code: u8,
    pub message: String,
    pub reconnect: bool,
}

impl Message for AccessDenied {
    const COMMAND_ID: u16 = TOCLIENT_ACCESS_DENIED;
}

impl DecodeMessage for AccessDenied {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);

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

        Ok(Self {
            code,
            message,
            reconnect: code == 6 || reconnect,
        })
    }
}

impl EncodeMessage for AccessDenied {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        let mut writer = PacketWriter::new();
        writer.write_u16(<Self as Message>::COMMAND_ID);
        writer.write_u8(self.code);
        writer.write_string16(&self.message)?;
        writer.write_u8(u8::from(self.reconnect));
        Ok(writer.into_inner())
    }
}

/// Reports an authentication failure using the legacy wide-string format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessDeniedLegacy {
    pub message: String,
}

impl Message for AccessDeniedLegacy {
    const COMMAND_ID: u16 = TOCLIENT_ACCESS_DENIED_LEGACY;
}

impl DecodeMessage for AccessDeniedLegacy {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);
        let message = reader.read_wide_string16()?;
        if !reader.is_empty() {
            return Err(ProtocolError::InvalidAuthPacket);
        }
        Ok(Self { message })
    }
}

/// Server challenge for the SRP authentication mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrpChallenge {
    pub salt: Vec<u8>,
    pub server_b: Vec<u8>,
}

impl Message for SrpChallenge {
    const COMMAND_ID: u16 = TOCLIENT_SRP_BYTES_SB;
}

impl DecodeMessage for SrpChallenge {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);
        let salt = reader.read_string16()?.to_vec();
        let server_b = reader.read_string16()?.to_vec();
        if !reader.is_empty() {
            return Err(ProtocolError::InvalidAuthPacket);
        }
        Ok(Self { salt, server_b })
    }
}

impl EncodeMessage for SrpChallenge {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        let mut writer = PacketWriter::new();
        writer.write_u16(<Self as Message>::COMMAND_ID);
        writer.write_binary_string16(&self.salt)?;
        writer.write_binary_string16(&self.server_b)?;
        Ok(writer.into_inner())
    }
}
