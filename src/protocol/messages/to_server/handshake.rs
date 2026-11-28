use crate::errors::ProtocolError;
use crate::protocol::commands::{
    TOSERVER_CLIENT_READY, TOSERVER_FIRST_SRP, TOSERVER_INIT, TOSERVER_INIT2, TOSERVER_SRP_BYTES_A,
    TOSERVER_SRP_BYTES_M,
};
use crate::protocol::constant::{
    MAX_PROTOCOL_VERSION, MIN_PROTOCOL_VERSION, SERIALIZATION_VERSION,
};
use crate::protocol::messages::{DecodeMessage, EncodeMessage, Message};
use crate::protocol::reader::PacketReader;
use crate::protocol::writer::PacketWriter;

const PLAYER_NAME_MAX_BYTES: usize = 20;

fn is_valid_player_name(player_name: &str) -> bool {
    !player_name.is_empty()
        && player_name.len() <= PLAYER_NAME_MAX_BYTES
        && player_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

/// Initial client handshake message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Init {
    pub player_name: String,
    pub protocol_min: u16,
    pub protocol_max: u16,
    pub serialization_version: u8,
}

impl Message for Init {
    const COMMAND_ID: u16 = TOSERVER_INIT;
}

impl EncodeMessage for Init {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        if !is_valid_player_name(&self.player_name) {
            return Err(ProtocolError::InvalidHello);
        }

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
        writer.write_u16(<Self as crate::protocol::messages::Message>::COMMAND_ID);
        writer.write_u8(self.serialization_version);
        writer.write_u16(0);
        writer.write_u16(self.protocol_min);
        writer.write_u16(self.protocol_max);
        writer.write_string16(&self.player_name)?;

        Ok(writer.into_inner())
    }
}

impl DecodeMessage for Init {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);

        let serialization_version = reader.read_u8()?;
        let _compression_modes = reader.read_u16()?;
        let protocol_min = reader.read_u16()?;
        let protocol_max = reader.read_u16()?;
        let player_name = std::str::from_utf8(reader.read_string16()?)
            .map_err(|_| ProtocolError::InvalidHello)?
            .to_owned();

        if !reader.is_empty() || protocol_min > protocol_max || !is_valid_player_name(&player_name)
        {
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
}

/// Acknowledges successful authentication and requests the initial world data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Init2 {
    pub language: String,
}

impl Message for Init2 {
    const COMMAND_ID: u16 = TOSERVER_INIT2;
}

impl EncodeMessage for Init2 {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        let mut writer = PacketWriter::new();
        writer.write_u16(<Self as crate::protocol::messages::Message>::COMMAND_ID);
        writer.write_string16(&self.language)?;
        Ok(writer.into_inner())
    }
}

impl DecodeMessage for Init2 {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);
        let language = if reader.is_empty() {
            String::new()
        } else {
            std::str::from_utf8(reader.read_string16()?)
                .map_err(|_| ProtocolError::InvalidHello)?
                .to_owned()
        };
        if !reader.is_empty() {
            return Err(ProtocolError::InvalidHello);
        }
        Ok(Self { language })
    }
}

/// Registers an account using the first-SRP authentication mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstSrp {
    pub salt: Vec<u8>,
    pub verifier: Vec<u8>,
    pub password_is_empty: bool,
}

impl Message for FirstSrp {
    const COMMAND_ID: u16 = TOSERVER_FIRST_SRP;
}

impl EncodeMessage for FirstSrp {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        let mut writer = PacketWriter::new();
        writer.write_u16(<Self as crate::protocol::messages::Message>::COMMAND_ID);
        writer.write_binary_string16(&self.salt)?;
        writer.write_binary_string16(&self.verifier)?;
        writer.write_u8(u8::from(self.password_is_empty));
        Ok(writer.into_inner())
    }
}

impl DecodeMessage for FirstSrp {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);
        let salt = reader.read_string16()?.to_vec();
        let verifier = reader.read_string16()?.to_vec();
        let password_is_empty = match reader.read_u8()? {
            0 => false,
            1 => true,
            _ => return Err(ProtocolError::InvalidAuthPacket),
        };
        if !reader.is_empty() {
            return Err(ProtocolError::InvalidAuthPacket);
        }
        Ok(Self {
            salt,
            verifier,
            password_is_empty,
        })
    }
}

/// Starts SRP authentication with the client's public value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrpBytesA {
    pub public_a: Vec<u8>,
    pub legacy_password: bool,
}

impl Message for SrpBytesA {
    const COMMAND_ID: u16 = TOSERVER_SRP_BYTES_A;
}

impl EncodeMessage for SrpBytesA {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        let mut writer = PacketWriter::new();
        writer.write_u16(<Self as crate::protocol::messages::Message>::COMMAND_ID);
        writer.write_binary_string16(&self.public_a)?;
        writer.write_u8(u8::from(!self.legacy_password));
        Ok(writer.into_inner())
    }
}

impl DecodeMessage for SrpBytesA {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);
        let public_a = reader.read_string16()?.to_vec();
        let legacy_password = match reader.read_u8()? {
            0 => true,
            1 => false,
            _ => return Err(ProtocolError::InvalidAuthPacket),
        };
        if !reader.is_empty() {
            return Err(ProtocolError::InvalidAuthPacket);
        }
        Ok(Self {
            public_a,
            legacy_password,
        })
    }
}

/// Completes SRP authentication with the client's proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrpBytesM {
    pub proof: Vec<u8>,
}

impl Message for SrpBytesM {
    const COMMAND_ID: u16 = TOSERVER_SRP_BYTES_M;
}

impl EncodeMessage for SrpBytesM {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        let mut writer = PacketWriter::new();
        writer.write_u16(<Self as crate::protocol::messages::Message>::COMMAND_ID);
        writer.write_binary_string16(&self.proof)?;
        Ok(writer.into_inner())
    }
}

impl DecodeMessage for SrpBytesM {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);
        let proof = reader.read_string16()?.to_vec();
        if !reader.is_empty() {
            return Err(ProtocolError::InvalidAuthPacket);
        }
        Ok(Self { proof })
    }
}

/// Announces that the client has loaded the initial definitions and media.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientReady {
    /// Client version major component.
    pub major: u8,
    /// Client version minor component.
    pub minor: u8,
    /// Client version patch component.
    pub patch: u8,
    /// Client version metadata string.
    pub version_string: String,
    /// Highest Formspec API version supported by the client.
    pub formspec_api_version: u16,
}

impl Message for ClientReady {
    const COMMAND_ID: u16 = TOSERVER_CLIENT_READY;
}

impl EncodeMessage for ClientReady {
    fn encode(&self) -> Result<bytes::Bytes, crate::errors::ProtocolError> {
        let mut writer = PacketWriter::new();
        writer.write_u16(<Self as crate::protocol::messages::Message>::COMMAND_ID);
        writer.write_u8(self.major);
        writer.write_u8(self.minor);
        writer.write_u8(self.patch);
        writer.write_u8(0);
        writer.write_string16(&self.version_string)?;
        writer.write_u16(self.formspec_api_version);
        Ok(writer.into_inner())
    }
}

impl DecodeMessage for ClientReady {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let mut reader = PacketReader::new(payload);
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        let patch = reader.read_u8()?;
        if reader.read_u8()? != 0 {
            return Err(ProtocolError::InvalidAuthPacket);
        }
        let version_string = std::str::from_utf8(reader.read_string16()?)
            .map_err(|_| ProtocolError::InvalidAuthPacket)?
            .to_owned();
        let formspec_api_version = if reader.is_empty() {
            1
        } else {
            reader.read_u16()?
        };
        if !reader.is_empty() {
            return Err(ProtocolError::InvalidAuthPacket);
        }
        Ok(Self {
            major,
            minor,
            patch,
            version_string,
            formspec_api_version,
        })
    }
}
