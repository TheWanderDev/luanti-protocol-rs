use bytes::Buf;
use memchr::memchr;

use crate::errors::ProtocolError;

#[derive(Debug)]
pub struct PacketReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> PacketReader<'a> {
    #[must_use]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn read_bytes(&mut self, length: usize) -> Result<&'a [u8], ProtocolError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(ProtocolError::Truncated)?;

        let value = self
            .data
            .get(self.offset..end)
            .ok_or(ProtocolError::Truncated)?;

        self.offset = end;

        Ok(value)
    }

    pub fn read_u8(&mut self) -> Result<u8, ProtocolError> {
        let mut bytes = self.read_bytes(1)?;
        bytes.try_get_u8().map_err(|_| ProtocolError::Truncated)
    }

    pub fn read_u16(&mut self) -> Result<u16, ProtocolError> {
        let mut bytes = self.read_bytes(2)?;
        bytes.try_get_u16().map_err(|_| ProtocolError::Truncated)
    }

    pub fn read_u32(&mut self) -> Result<u32, ProtocolError> {
        let mut bytes = self.read_bytes(4)?;
        bytes.try_get_u32().map_err(|_| ProtocolError::Truncated)
    }

    pub fn read_string16(&mut self) -> Result<&'a [u8], ProtocolError> {
        let length = usize::from(self.read_u16()?);
        self.read_bytes(length)
    }

    pub fn read_len32(&mut self) -> Result<usize, ProtocolError> {
        usize::try_from(self.read_u32()?).map_err(|_| ProtocolError::InvalidStringLength)
    }

    pub fn read_string32(&mut self) -> Result<&'a [u8], ProtocolError> {
        let length = self.read_len32()?;
        self.read_bytes(length)
    }

    pub fn read_line(&mut self) -> Result<Option<&'a [u8]>, ProtocolError> {
        let remaining = self
            .data
            .get(self.offset..)
            .ok_or(ProtocolError::Truncated)?;

        let length = memchr(b'\n', remaining).map_or(remaining.len(), |newline| newline + 1);

        if length == 0 {
            return Ok(None);
        }

        let line = self.read_bytes(length)?;
        Ok(Some(line))
    }

    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len() - self.offset
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }
    pub fn read_u64(&mut self) -> Result<u64, ProtocolError> {
        let mut bytes = self.read_bytes(8)?;
        bytes.try_get_u64().map_err(|_| ProtocolError::Truncated)
    }

    pub fn read_wide_string16(&mut self) -> Result<String, ProtocolError> {
        let length = usize::from(self.read_u16()?);
        let byte_length = length.checked_mul(2).ok_or(ProtocolError::Truncated)?;

        let bytes = self.read_bytes(byte_length)?;

        std::char::decode_utf16(
            bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|chunk| u16::from_be_bytes(*chunk)),
        )
        .collect::<Result<String, _>>()
        .map_err(|_| ProtocolError::InvalidWideString)
    }
    pub fn read_i32(&mut self) -> Result<i32, ProtocolError> {
        let mut bytes = self.read_bytes(4)?;
        bytes.try_get_i32().map_err(|_| ProtocolError::Truncated)
    }

    pub fn read_f32(&mut self) -> Result<f32, ProtocolError> {
        Ok(f32::from_bits(self.read_u32()?))
    }
    pub fn read_i16(&mut self) -> Result<i16, ProtocolError> {
        let mut bytes = self.read_bytes(2)?;
        bytes.try_get_i16().map_err(|_| ProtocolError::Truncated)
    }

    pub fn read_f1000(&mut self) -> Result<f32, ProtocolError> {
        Ok(self.read_i32()? as f32 / 1000.0)
    }
}
