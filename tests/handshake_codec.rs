use luanti_protocol_rs::protocol::commands::ToServerCommand;
use luanti_protocol_rs::protocol::constant::{
    AUTH_MECHANISM_LEGACY_PASSWORD, AUTH_MECHANISM_SRP, MAX_PROTOCOL_VERSION, MIN_PROTOCOL_VERSION,
    SERIALIZATION_VERSION,
};
use luanti_protocol_rs::protocol::messages::{DecodeMessage, EncodeMessage};
use luanti_protocol_rs::{to_client::Hello, to_server::Init};

#[test]
fn init_message_matches_wire_layout_and_round_trips() {
    let message = Init {
        player_name: "tester".to_owned(),
        protocol_min: MIN_PROTOCOL_VERSION,
        protocol_max: MAX_PROTOCOL_VERSION,
        serialization_version: SERIALIZATION_VERSION,
    };

    let encoded_result = message.encode();
    assert!(encoded_result.is_ok());
    let encoded = encoded_result.unwrap_or_default();

    let mut expected_prefix = vec![0x00, 0x02, SERIALIZATION_VERSION, 0x00, 0x00];
    expected_prefix.extend_from_slice(&MIN_PROTOCOL_VERSION.to_be_bytes());
    expected_prefix.extend_from_slice(&MAX_PROTOCOL_VERSION.to_be_bytes());
    assert_eq!(
        &encoded[..expected_prefix.len()],
        expected_prefix.as_slice()
    );
    assert_eq!(Init::decode(&encoded), Ok(message));
}

#[test]
fn hello_message_matches_wire_layout_and_round_trips() {
    let message = Hello {
        serialization_version: SERIALIZATION_VERSION,
        protocol_version: MAX_PROTOCOL_VERSION,
        auth_mechanisms: AUTH_MECHANISM_LEGACY_PASSWORD | AUTH_MECHANISM_SRP,
    };

    let encoded_result = message.encode();
    assert!(encoded_result.is_ok());
    let encoded = encoded_result.unwrap_or_default();

    let mut expected = vec![0x00, 0x02, SERIALIZATION_VERSION, 0x00, 0x00];
    expected.extend_from_slice(&MAX_PROTOCOL_VERSION.to_be_bytes());
    expected.extend_from_slice(&message.auth_mechanisms.to_be_bytes());
    expected.extend_from_slice(&[0x00, 0x00]);
    assert_eq!(encoded.as_ref(), expected.as_slice());
    assert_eq!(Hello::decode(&encoded), Ok(message));
}

#[test]
fn handshake_decoders_reject_trailing_bytes() {
    let init = Init {
        player_name: "tester".to_owned(),
        protocol_min: MIN_PROTOCOL_VERSION,
        protocol_max: MAX_PROTOCOL_VERSION,
        serialization_version: SERIALIZATION_VERSION,
    };
    let mut init_bytes = init.encode().unwrap_or_default().to_vec();
    init_bytes.push(0);
    assert!(Init::decode(&init_bytes).is_err());

    let hello = Hello {
        serialization_version: SERIALIZATION_VERSION,
        protocol_version: MAX_PROTOCOL_VERSION,
        auth_mechanisms: AUTH_MECHANISM_SRP,
    };
    let mut hello_bytes = hello.encode().unwrap_or_default().to_vec();
    hello_bytes.push(0);
    assert!(Hello::decode(&hello_bytes).is_err());
}

#[test]
fn client_commands_use_expected_channels_and_reliability() {
    assert_eq!(ToServerCommand::Init.channel(), 1);
    assert!(!ToServerCommand::Init.is_reliable());

    assert_eq!(ToServerCommand::PlayerPosition.channel(), 0);
    assert!(!ToServerCommand::PlayerPosition.is_reliable());

    assert_eq!(ToServerCommand::ChatMessage.channel(), 0);
    assert!(ToServerCommand::ChatMessage.is_reliable());

    assert_eq!(ToServerCommand::ClientReady.channel(), 1);
    assert!(ToServerCommand::ClientReady.is_reliable());

    assert_eq!(ToServerCommand::GotBlocks.channel(), 2);
    assert!(ToServerCommand::GotBlocks.is_reliable());
}
