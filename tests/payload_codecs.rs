use luanti_protocol_rs::errors::ProtocolError;
use luanti_protocol_rs::protocol::chat::{
    ChatMessage, ChatMessageType, decode_chat_message, encode_chat_message,
};
use luanti_protocol_rs::protocol::inventory_action::{
    InventoryAction, InventoryLocation, encode_inventory_action,
};
use luanti_protocol_rs::protocol::status::{
    decode_movement_parameters, decode_player_hp, decode_time_of_day,
};

#[test]
fn chat_decoder_matches_wire_fixture() {
    let fixture = [
        1, 1, 0, 1, 0, b'A', 0, 2, 0, b'H', 0, b'i', 0, 0, 0, 0, 0, 0, 0x04, 0xD2,
    ];
    assert_eq!(
        decode_chat_message(&fixture),
        Ok(ChatMessage {
            message_type: ChatMessageType::Normal,
            sender: "A".to_owned(),
            message: "Hi".to_owned(),
            timestamp: 1234,
        })
    );
}

#[test]
fn chat_encoder_matches_server_command_fixture() {
    let encoded = encode_chat_message("Hi");
    assert_eq!(
        encoded.as_deref(),
        Ok(&[0, 0x32, 0, 2, 0, b'H', 0, b'i'][..])
    );
}

#[test]
fn chat_decoder_rejects_unknown_versions_and_trailing_data() {
    assert_eq!(
        decode_chat_message(&[2, 1]),
        Err(ProtocolError::InvalidChatMessageVersion(2))
    );

    let mut fixture = vec![1, 1, 0, 0, 0, 0];
    fixture.extend_from_slice(&0_u64.to_be_bytes());
    fixture.push(0xFF);
    assert_eq!(
        decode_chat_message(&fixture),
        Err(ProtocolError::InvalidChatMessage)
    );
}

#[test]
fn inventory_action_encoder_matches_wire_fixture() {
    let action = InventoryAction::Move {
        count: 2,
        from_inv: InventoryLocation::CurrentPlayer,
        from_list: "main".to_owned(),
        from_index: 3,
        to_inv: InventoryLocation::CurrentPlayer,
        to_list: "craft".to_owned(),
        to_index: 1,
    };
    let encoded = encode_inventory_action(&action);
    assert_eq!(
        encoded.as_deref(),
        Ok(&[
            0, 0x31, b'M', b'o', b'v', b'e', b' ', b'2', b' ', b'c', b'u', b'r', b'r', b'e', b'n',
            b't', b'_', b'p', b'l', b'a', b'y', b'e', b'r', b' ', b'm', b'a', b'i', b'n', b' ',
            b'3', b' ', b'c', b'u', b'r', b'r', b'e', b'n', b't', b'_', b'p', b'l', b'a', b'y',
            b'e', b'r', b' ', b'c', b'r', b'a', b'f', b't', b' ', b'1',
        ][..])
    );
}

#[test]
fn inventory_action_encoder_rejects_negative_indices_and_invalid_tokens() {
    let invalid_index = InventoryAction::Drop {
        count: 1,
        from_inv: InventoryLocation::CurrentPlayer,
        from_list: "main".to_owned(),
        from_index: -1,
    };
    assert!(matches!(
        encode_inventory_action(&invalid_index),
        Err(ProtocolError::InvalidPlayerValue(_))
    ));

    let invalid_token = InventoryAction::Drop {
        count: 1,
        from_inv: InventoryLocation::CurrentPlayer,
        from_list: "main list".to_owned(),
        from_index: 0,
    };
    assert!(matches!(
        encode_inventory_action(&invalid_token),
        Err(ProtocolError::InvalidPlayerValue(_))
    ));
}

#[test]
fn status_decoders_validate_payload_length_and_values() {
    let mut time_of_day = 24_001_u16.to_be_bytes().to_vec();
    time_of_day.extend_from_slice(&1.25_f32.to_be_bytes());
    let decoded = decode_time_of_day(&time_of_day);
    assert_eq!(
        decoded.as_ref().map(|value| (value.time, value.speed)),
        Ok((1, 1.25))
    );

    assert_eq!(decode_time_of_day(&[0]), Err(ProtocolError::Truncated));
    assert_eq!(
        decode_player_hp(&[0, 20, 2]),
        Err(ProtocolError::InvalidPlayerPacket)
    );
    assert_eq!(
        decode_player_hp(&[0, 20, 1]).map(|value| value.damage_effect),
        Ok(true)
    );

    let mut movement = Vec::new();
    for value in 0_u8..12 {
        movement.extend_from_slice(&f32::from(value).to_be_bytes());
    }
    assert!(decode_movement_parameters(&movement).is_ok());
    movement.push(0);
    assert_eq!(
        decode_movement_parameters(&movement),
        Err(ProtocolError::InvalidPlayerPacket)
    );
}
