use crate::errors::ProtocolError;
use crate::protocol::reader::PacketReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimapMode {
    pub mode_type: u16,
    pub label: String,
    pub size: u16,
    pub texture: String,
    pub scale: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimapModesUpdate {
    pub modes: Vec<MinimapMode>,
    pub selected_mode: u16,
}

pub(crate) fn decode_minimap_modes(data: &[u8]) -> Result<MinimapModesUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let mode_count = usize::from(reader.read_u16()?);
    let selected_mode = reader.read_u16()?;

    if mode_count > reader.remaining() / 10 {
        return Err(ProtocolError::InvalidMinimap(
            "mode count exceeds packet data",
        ));
    }

    let mut modes = Vec::with_capacity(mode_count);

    for _ in 0..mode_count {
        let mode_type = reader.read_u16()?;
        let label = decode_string(reader.read_string16()?);
        let size = reader.read_u16()?;
        let texture = decode_string(reader.read_string16()?);
        let scale = reader.read_u16()?;

        modes.push(MinimapMode {
            mode_type,
            label,
            size,
            texture,
            scale,
        });
    }

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidMinimap("packet has trailing bytes"));
    }

    Ok(MinimapModesUpdate {
        modes,
        selected_mode,
    })
}

fn decode_string(value: &[u8]) -> String {
    String::from_utf8_lossy(value).into_owned()
}
