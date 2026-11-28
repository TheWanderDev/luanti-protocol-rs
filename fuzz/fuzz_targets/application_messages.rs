#![no_main]

use libfuzzer_sys::fuzz_target;
use luanti_protocol_rs::protocol::constant::{MAX_PROTOCOL_VERSION, SERIALIZATION_VERSION};
use luanti_protocol_rs::protocol::messages::{
    DecodeMessage, DecodeMessageWithProtocolVersion, DecodeMessageWithSerializationVersion,
    command_id,
};
use luanti_protocol_rs::{to_client, to_server};

fuzz_target!(|data: &[u8]| {
    if data.len() > 1_048_576 {
        return;
    }

    let _ = command_id(data);
    let _ = to_server::Init::decode(data);
    let _ = to_client::Hello::decode(data);
    let _ = to_client::InventoryUpdate::decode(data, MAX_PROTOCOL_VERSION);
    let _ = to_client::BlockDataUpdate::decode(data, SERIALIZATION_VERSION);
});
