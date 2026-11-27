use bytes::Bytes;
use luanti_protocol_rs::errors::ProtocolError;
use luanti_protocol_rs::protocol::constant::{MTP_MAX_PACKET_SIZE, PROTOCOL_ID};
use luanti_protocol_rs::protocol::packet::{
    ControlPacket, DecodedDatagram, MtpPayload, decode_datagram, encode_ack, encode_disconnect,
    encode_original, encode_ping, encode_reliable, encode_reliable_split, encode_set_peer_id,
};

#[test]
fn original_packet_matches_wire_fixture() {
    let mut fixture = PROTOCOL_ID.to_be_bytes().to_vec();
    fixture.extend_from_slice(&0x1234_u16.to_be_bytes());
    fixture.push(2);
    fixture.extend_from_slice(&[1, 0xAA, 0xBB]);

    let decoded = decode_datagram(Bytes::from(fixture));
    assert_eq!(
        decoded,
        Ok(DecodedDatagram {
            sender_peer_id: 0x1234,
            channel: 2,
            reliable_sequence: None,
            payload: MtpPayload::Original(Bytes::from_static(&[0xAA, 0xBB])),
        })
    );
}

#[test]
fn original_encoder_emits_expected_wire_bytes() {
    let mut expected = PROTOCOL_ID.to_be_bytes().to_vec();
    expected.extend_from_slice(&0x1234_u16.to_be_bytes());
    expected.extend_from_slice(&[1, 1, 0xAA, 0xBB]);

    assert_eq!(
        encode_original(0x1234, 1, &[0xAA, 0xBB]).as_ref(),
        expected.as_slice()
    );
}

#[test]
fn control_packet_encoders_match_wire_fixtures() {
    let mut ack = PROTOCOL_ID.to_be_bytes().to_vec();
    ack.extend_from_slice(&[0, 7, 0, 0, 0, 0xBE, 0xEF]);
    assert_eq!(encode_ack(7, 0, 0xBEEF).as_ref(), ack.as_slice());

    let mut ping = PROTOCOL_ID.to_be_bytes().to_vec();
    ping.extend_from_slice(&[0, 7, 0, 0, 2]);
    assert_eq!(encode_ping(7, 0).as_ref(), ping.as_slice());

    let mut disconnect = PROTOCOL_ID.to_be_bytes().to_vec();
    disconnect.extend_from_slice(&[0, 7, 0, 0, 3]);
    assert_eq!(encode_disconnect(7, 0).as_ref(), disconnect.as_slice());

    let mut set_peer_id = PROTOCOL_ID.to_be_bytes().to_vec();
    set_peer_id.extend_from_slice(&[0, 7, 0, 0, 1, 0x12, 0x34]);
    assert_eq!(
        encode_set_peer_id(7, 0, 0x1234).as_ref(),
        set_peer_id.as_slice()
    );
}

#[test]
fn control_packets_decode_ack_ping_and_disconnect() {
    let packets = [
        (encode_ack(1, 0, 0xCAFE), ControlPacket::Ack(0xCAFE)),
        (
            encode_set_peer_id(1, 0, 0x1234),
            ControlPacket::SetPeerId(0x1234),
        ),
        (encode_ping(1, 0), ControlPacket::Ping),
        (encode_disconnect(1, 0), ControlPacket::Disconnect),
    ];

    for (packet, expected) in packets {
        let decoded = decode_datagram(packet);
        assert_eq!(
            decoded.map(|value| value.payload),
            Ok(MtpPayload::Control(expected))
        );
    }
}

#[test]
fn reliable_original_preserves_sequence_and_payload() {
    let decoded = decode_datagram(encode_reliable(9, 1, 0x4567, &[1, 2, 3]));
    assert_eq!(
        decoded,
        Ok(DecodedDatagram {
            sender_peer_id: 9,
            channel: 1,
            reliable_sequence: Some(0x4567),
            payload: MtpPayload::Original(Bytes::from_static(&[1, 2, 3])),
        })
    );
}

#[test]
fn reliable_split_preserves_chunk_metadata() {
    let decoded = decode_datagram(encode_reliable_split(4, 2, 10, 20, 3, 1, &[7, 8]));
    assert_eq!(
        decoded,
        Ok(DecodedDatagram {
            sender_peer_id: 4,
            channel: 2,
            reliable_sequence: Some(10),
            payload: MtpPayload::Split {
                sequence: 20,
                chunk_count: 3,
                chunk_number: 1,
                data: Bytes::from_static(&[7, 8]),
            },
        })
    );
}

#[test]
fn decoder_rejects_truncated_and_invalid_headers() {
    assert_eq!(
        decode_datagram(Bytes::from_static(&[0; 7])),
        Err(ProtocolError::Truncated)
    );

    let mut invalid_id = encode_original(1, 0, &[]).to_vec();
    invalid_id[0] ^= 0xFF;
    assert!(matches!(
        decode_datagram(Bytes::from(invalid_id)),
        Err(ProtocolError::InvalidProtocolId(_))
    ));

    let mut invalid_channel = PROTOCOL_ID.to_be_bytes().to_vec();
    invalid_channel.extend_from_slice(&[0, 1, 3, 1]);
    assert_eq!(
        decode_datagram(Bytes::from(invalid_channel)),
        Err(ProtocolError::InvalidChannel(3))
    );
}

#[test]
fn decoder_rejects_invalid_control_and_split_frames() {
    let mut invalid_control = PROTOCOL_ID.to_be_bytes().to_vec();
    invalid_control.extend_from_slice(&[0, 1, 0, 0, 9]);
    assert_eq!(
        decode_datagram(Bytes::from(invalid_control)),
        Err(ProtocolError::InvalidControlType(9))
    );

    let mut invalid_split = PROTOCOL_ID.to_be_bytes().to_vec();
    invalid_split.extend_from_slice(&[0, 1, 0, 2, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        decode_datagram(Bytes::from(invalid_split)),
        Err(ProtocolError::InvalidSplitPacket)
    );
}

#[test]
fn decoder_rejects_oversized_datagrams() {
    let mut packet = encode_original(1, 0, &[]).to_vec();
    packet.resize(MTP_MAX_PACKET_SIZE + 1, 0);
    assert_eq!(
        decode_datagram(Bytes::from(packet)),
        Err(ProtocolError::PacketTooLarge)
    );
}

#[test]
fn decoder_rejects_invalid_channel_in_encoded_datagram() {
    let packet = encode_original(1, 3, &[1]);
    assert_eq!(
        decode_datagram(packet),
        Err(ProtocolError::InvalidChannel(3))
    );
}

#[test]
fn decoder_rejects_nested_reliable_and_invalid_split_indices() {
    let mut nested_reliable = PROTOCOL_ID.to_be_bytes().to_vec();
    nested_reliable.extend_from_slice(&[0, 1, 0, 3, 0, 1, 3, 0, 2, 1]);
    assert_eq!(
        decode_datagram(Bytes::from(nested_reliable)),
        Err(ProtocolError::InvalidReliablePacket)
    );

    let mut invalid_split_index = PROTOCOL_ID.to_be_bytes().to_vec();
    invalid_split_index.extend_from_slice(&[0, 1, 0, 2, 0, 1, 0, 2, 0, 2]);
    assert_eq!(
        decode_datagram(Bytes::from(invalid_split_index)),
        Err(ProtocolError::InvalidSplitPacket)
    );
}
