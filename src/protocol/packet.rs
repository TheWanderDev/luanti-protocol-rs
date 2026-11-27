use crate::errors::ProtocolError;
use crate::protocol::constant::{
    BASE_HEADER_SIZE, CHANNEL_COUNT, MAX_SPLIT_CHUNKS, MTP_MAX_PACKET_SIZE, PROTOCOL_ID,
};
use bytes::{BufMut, Bytes, BytesMut};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MtpPacketType {
    Control = 0,
    Original = 1,
    Split = 2,
    Reliable = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MtpControlType {
    Ack = 0,
    SetPeerId = 1,
    Ping = 2,
    Disconnect = 3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlPacket {
    Ack(u16),
    SetPeerId(u16),
    Ping,
    Disconnect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MtpPayload {
    Control(ControlPacket),
    Original(Bytes),
    Split {
        sequence: u16,
        chunk_count: u16,
        chunk_number: u16,
        data: Bytes,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedDatagram {
    pub sender_peer_id: u16,
    pub channel: u8,
    pub reliable_sequence: Option<u16>,
    pub payload: MtpPayload,
}

#[must_use]
pub fn encode_header(peer_id: u16, channel: u8) -> BytesMut {
    let mut output = BytesMut::with_capacity(BASE_HEADER_SIZE);

    output.put_u32(PROTOCOL_ID);
    output.put_u16(peer_id);
    output.put_u8(channel);

    output
}

fn decode_body(
    data: Bytes,
    inside_reliable: bool,
) -> Result<(Option<u16>, MtpPayload), ProtocolError> {
    let raw_type = data.first().copied().ok_or(ProtocolError::Truncated)?;

    let packet_type = MtpPacketType::try_from(raw_type)
        .map_err(|_| ProtocolError::UnsupportedPacketType(raw_type))?;

    match packet_type {
        MtpPacketType::Original => Ok((None, MtpPayload::Original(data.slice(1..)))),

        MtpPacketType::Control => {
            let raw_control = data.get(1).copied().ok_or(ProtocolError::Truncated)?;

            let control_type = MtpControlType::try_from(raw_control)
                .map_err(|_| ProtocolError::InvalidControlType(raw_control))?;

            let control = match control_type {
                MtpControlType::Ack | MtpControlType::SetPeerId => {
                    if data.len() != 4 {
                        return Err(ProtocolError::Truncated);
                    }

                    let value = u16::from_be_bytes([data[2], data[3]]);

                    if control_type == MtpControlType::Ack {
                        ControlPacket::Ack(value)
                    } else {
                        ControlPacket::SetPeerId(value)
                    }
                }

                MtpControlType::Ping | MtpControlType::Disconnect => {
                    if data.len() != 2 {
                        return Err(ProtocolError::Truncated);
                    }

                    if control_type == MtpControlType::Ping {
                        ControlPacket::Ping
                    } else {
                        ControlPacket::Disconnect
                    }
                }
            };

            Ok((None, MtpPayload::Control(control)))
        }

        MtpPacketType::Split => {
            if data.len() < 7 {
                return Err(ProtocolError::Truncated);
            }

            let sequence = u16::from_be_bytes([data[1], data[2]]);
            let chunk_count = u16::from_be_bytes([data[3], data[4]]);
            let chunk_number = u16::from_be_bytes([data[5], data[6]]);

            if chunk_count == 0
                || chunk_count > MAX_SPLIT_CHUNKS
                || chunk_number >= chunk_count
            {
                return Err(ProtocolError::InvalidSplitPacket);
            }

            Ok((
                None,
                MtpPayload::Split {
                    sequence,
                    chunk_count,
                    chunk_number,
                    data: data.slice(7..),
                },
            ))
        }

        MtpPacketType::Reliable => {
            if inside_reliable || data.len() < 4 {
                return Err(ProtocolError::InvalidReliablePacket);
            }

            let sequence = u16::from_be_bytes([data[1], data[2]]);
            let (nested_sequence, payload) = decode_body(data.slice(3..), true)?;

            if nested_sequence.is_some() {
                return Err(ProtocolError::InvalidReliablePacket);
            }

            Ok((Some(sequence), payload))
        }
    }
}
pub fn decode_datagram(data: Bytes) -> Result<DecodedDatagram, ProtocolError> {
    if data.len() > MTP_MAX_PACKET_SIZE {
        return Err(ProtocolError::PacketTooLarge);
    }

    if data.len() < BASE_HEADER_SIZE + 1 {
        return Err(ProtocolError::Truncated);
    }

    let protocol_id = u32::from_be_bytes(
        data[0..4]
            .try_into()
            .map_err(|_| ProtocolError::Truncated)?,
    );

    if protocol_id != PROTOCOL_ID {
        return Err(ProtocolError::InvalidProtocolId(protocol_id));
    }

    let sender_peer_id = u16::from_be_bytes([data[4], data[5]]);
    let channel = data[6];

    if usize::from(channel) >= CHANNEL_COUNT {
        return Err(ProtocolError::InvalidChannel(channel));
    }

    let body = data.slice(BASE_HEADER_SIZE..);
    let (reliable_sequence, payload) = decode_body(body, false)?;

    Ok(DecodedDatagram {
        sender_peer_id,
        channel,
        reliable_sequence,
        payload,
    })
}

#[must_use]
pub fn encode_reliable_split(
    peer_id: u16,
    channel: u8,
    reliable_sequence: u16,
    split_sequence: u16,
    chunk_count: u16,
    chunk_number: u16,
    data: &[u8],
) -> Bytes {
    let mut output = encode_header(peer_id, channel);

    output.put_u8(u8::from(MtpPacketType::Reliable));
    output.put_u16(reliable_sequence);
    output.put_u8(u8::from(MtpPacketType::Split));
    output.put_u16(split_sequence);
    output.put_u16(chunk_count);
    output.put_u16(chunk_number);
    output.extend_from_slice(data);

    output.freeze()
}

#[must_use]
pub fn encode_ack(peer_id: u16, channel: u8, sequence: u16) -> Bytes {
    let mut output = encode_header(peer_id, channel);

    output.put_u8(u8::from(MtpPacketType::Control));
    output.put_u8(u8::from(MtpControlType::Ack));
    output.put_u16(sequence);

    output.freeze()
}

#[must_use]
pub fn encode_set_peer_id(server_peer_id: u16, channel: u8, assigned_peer_id: u16) -> Bytes {
    let mut output = encode_header(server_peer_id, channel);

    output.put_u8(u8::from(MtpPacketType::Control));
    output.put_u8(u8::from(MtpControlType::SetPeerId));
    output.put_u16(assigned_peer_id);

    output.freeze()
}

#[must_use]
pub fn encode_ping(peer_id: u16, channel: u8) -> Bytes {
    let mut output = encode_header(peer_id, channel);

    output.put_u8(u8::from(MtpPacketType::Control));
    output.put_u8(u8::from(MtpControlType::Ping));

    output.freeze()
}

#[must_use]
pub fn encode_original(peer_id: u16, channel: u8, payload: &[u8]) -> Bytes {
    let mut output = encode_header(peer_id, channel);

    output.put_u8(u8::from(MtpPacketType::Original));
    output.extend_from_slice(payload);

    output.freeze()
}

#[must_use]
pub fn encode_reliable(peer_id: u16, channel: u8, sequence: u16, payload: &[u8]) -> Bytes {
    let mut output = encode_header(peer_id, channel);

    output.put_u8(u8::from(MtpPacketType::Reliable));
    output.put_u16(sequence);
    output.put_u8(u8::from(MtpPacketType::Original));
    output.extend_from_slice(payload);

    output.freeze()
}

#[must_use]
pub fn encode_disconnect(peer_id: u16, channel: u8) -> Bytes {
    let mut output = encode_header(peer_id, channel);

    output.put_u8(u8::from(MtpPacketType::Control));
    output.put_u8(u8::from(MtpControlType::Disconnect));

    output.freeze()
}
