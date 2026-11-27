use bytes::{BufMut, Bytes, BytesMut};

use crate::errors::ProtocolError;

#[derive(Debug, Default)]
pub struct PacketWriter {
    buffer: BytesMut,
}

impl PacketWriter {
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(64)
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
        }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.buffer.put_u8(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.buffer.put_u16(value);
    }

    pub fn write_u32(&mut self, value: u32) {
        self.buffer.put_u32(value);
    }

    pub fn write_i16(&mut self, value: i16) {
        self.buffer.put_i16(value);
    }

    pub fn write_string16(&mut self, value: &str) -> Result<(), ProtocolError> {
        let length = u16::try_from(value.len()).map_err(|_| ProtocolError::InvalidStringLength)?;

        self.write_u16(length);
        self.buffer.put_slice(value.as_bytes());

        Ok(())
    }

    pub fn write_string32(&mut self, value: &str) -> Result<(), ProtocolError> {
        let length = u32::try_from(value.len()).map_err(|_| ProtocolError::InvalidStringLength)?;

        self.write_u32(length);
        self.buffer.put_slice(value.as_bytes());

        Ok(())
    }

    pub fn write_bytes(&mut self, value: &[u8]) {
        self.buffer.put_slice(value);
    }

    pub fn write_binary_string16(&mut self, value: &[u8]) -> Result<(), ProtocolError> {
        let length = u16::try_from(value.len()).map_err(|_| ProtocolError::InvalidStringLength)?;

        self.write_u16(length);
        self.write_bytes(value);

        Ok(())
    }

    #[must_use]
    pub fn into_inner(self) -> Bytes {
        self.buffer.freeze()
    }
    pub fn write_u64(&mut self, value: u64) {
        self.buffer.put_u64(value);
    }

    pub fn write_wide_string16(&mut self, value: &str) -> Result<(), ProtocolError> {
        let length = u16::try_from(value.encode_utf16().count())
            .map_err(|_| ProtocolError::InvalidStringLength)?;

        self.write_u16(length);
        self.buffer.reserve(usize::from(length) * 2);

        for unit in value.encode_utf16() {
            self.write_u16(unit);
        }

        Ok(())
    }
    pub fn write_i32(&mut self, value: i32) {
        self.buffer.put_i32(value);
    }

    pub fn write_f32(&mut self, value: f32) {
        self.buffer.put_f32(value);
    }
}
