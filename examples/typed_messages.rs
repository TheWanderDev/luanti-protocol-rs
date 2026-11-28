use luanti_protocol_rs::protocol::messages::{DecodeMessage, EncodeMessage, command_id};
use luanti_protocol_rs::{to_client, to_server};
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .with_target(false)
        .init();

    let init = to_server::Init {
        player_name: String::from("player"),
        protocol_min: 37,
        protocol_max: 55,
        serialization_version: 29,
    };

    let init_bytes = init.encode()?;
    info!(
        command_id = command_id(&init_bytes)?,
        encoded_bytes = init_bytes.len(),
        player = %init.player_name,
        protocol_min = init.protocol_min,
        protocol_max = init.protocol_max,
        serialization_version = init.serialization_version,
        "client handshake encoded"
    );

    let outgoing_chat = to_server::ChatMessage {
        message: String::from("Hello server"),
    };

    let chat_bytes = outgoing_chat.encode()?;
    let decoded_chat = to_server::ChatMessage::decode(&chat_bytes)?;

    info!(
        command_id = command_id(&chat_bytes)?,
        encoded_bytes = chat_bytes.len(),
        message = %decoded_chat.message,
        "client chat encoded and decoded"
    );

    let incoming_hello = to_client::Hello {
        serialization_version: 29,
        protocol_version: 55,
        auth_mechanisms: 1,
    };

    let hello_bytes = incoming_hello.encode()?;
    let decoded_hello = to_client::Hello::decode(&hello_bytes)?;

    info!(
        command_id = command_id(&hello_bytes)?,
        encoded_bytes = hello_bytes.len(),
        serialization_version = decoded_hello.serialization_version,
        protocol_version = decoded_hello.protocol_version,
        auth_mechanisms = decoded_hello.auth_mechanisms,
        "server handshake encoded and decoded"
    );
    info!("running example test was successful");
    Ok(())
}
