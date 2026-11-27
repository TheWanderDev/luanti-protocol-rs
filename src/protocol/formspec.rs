use crate::errors::ProtocolError;
use crate::protocol::commands::ToServerCommand;
use crate::protocol::constant::MAX_SPLIT_PACKET_PAYLOAD_SIZE;
use crate::protocol::reader::PacketReader;
use crate::protocol::writer::PacketWriter;
use bytes::Bytes;
use glam::I16Vec3;
use std::cell::Cell;
use winnow::combinator::delimited;
use winnow::prelude::*;
use winnow::token::take_while;

pub const MAX_FORMSPEC_BYTES: usize = MAX_SPLIT_PACKET_PAYLOAD_SIZE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormspecElement {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormspecDocument {
    pub elements: Vec<FormspecElement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryFormspec {
    pub formspec: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowFormspec {
    pub formspec: String,
    pub form_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetachedInventoryUpdate {
    pub name: String,
    pub keep_inventory: bool,
    pub serialized_inventory: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormspecPrepend {
    pub formspec: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormspecField {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryFields {
    pub form_name: String,
    pub fields: Vec<FormspecField>,
}

fn read_submitted_fields(
    reader: &mut PacketReader<'_>,
) -> Result<Vec<FormspecField>, ProtocolError> {
    let count = usize::from(reader.read_u16()?);
    if count > reader.remaining() / 6 {
        return Err(ProtocolError::InvalidFormspec(
            "submitted field count exceeds packet size",
        ));
    }

    let mut fields = Vec::with_capacity(count);
    for _ in 0..count {
        fields.push(FormspecField {
            name: read_string16(reader)?,
            value: read_string32(reader)?,
        });
    }

    Ok(fields)
}

pub fn decode_inventory_fields(data: &[u8]) -> Result<InventoryFields, ProtocolError> {
    let mut reader = PacketReader::new(data);
    if reader.read_u16()? != u16::from(ToServerCommand::InventoryFields) {
        return Err(ProtocolError::InvalidFormspec(
            "invalid inventory fields command",
        ));
    }

    let form_name = read_string16(&mut reader)?;
    let fields = read_submitted_fields(&mut reader)?;
    if !reader.is_empty() {
        return Err(ProtocolError::InvalidFormspec(
            "unexpected data after inventory fields",
        ));
    }

    Ok(InventoryFields { form_name, fields })
}

fn split_escaped_arguments(source: &str) -> Vec<&str> {
    let mut arguments = Vec::new();
    let mut start = 0;
    let mut escaped = false;

    for (index, character) in source.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }

        if character == '\\' {
            escaped = true;
        } else if character == ';' {
            arguments.push(&source[start..index]);
            start = index + character.len_utf8();
        }
    }

    arguments.push(&source[start..]);
    arguments
}

fn parse_element(input: &mut &str) -> ModalResult<FormspecElement> {
    let name = take_while(1.., |character: char| {
        character.is_ascii_alphanumeric() || character == '_'
    })
    .parse_next(input)?;
    let escaped = Cell::new(false);
    let raw_arguments = delimited(
        '[',
        take_while(0.., |character: char| {
            if escaped.replace(false) {
                true
            } else if character == '\\' {
                escaped.set(true);
                true
            } else {
                character != ']'
            }
        }),
        ']',
    )
    .parse_next(input)?;
    let arguments = if raw_arguments.is_empty() {
        Vec::new()
    } else {
        split_escaped_arguments(raw_arguments)
    };

    Ok(FormspecElement {
        name: name.to_owned(),
        arguments: arguments.into_iter().map(str::to_owned).collect(),
    })
}

fn parse_whitespace<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_while(0.., |character: char| character.is_whitespace()).parse_next(input)
}

impl FormspecDocument {
    pub fn parse(source: &str) -> Result<Self, ProtocolError> {
        if source.len() > MAX_FORMSPEC_BYTES {
            return Err(ProtocolError::InvalidFormspec(
                "formspec exceeds size limit",
            ));
        }

        let mut input = source;
        let mut elements = Vec::new();

        while !input.is_empty() {
            parse_whitespace(&mut input)
                .map_err(|_| ProtocolError::InvalidFormspec("invalid formspec spacing"))?;
            if input.is_empty() {
                break;
            }
            let previous_length = input.len();
            let element = parse_element(&mut input)
                .map_err(|_| ProtocolError::InvalidFormspec("invalid formspec element"))?;

            if input.len() >= previous_length {
                return Err(ProtocolError::InvalidFormspec(
                    "formspec parser made no progress",
                ));
            }

            elements.push(element);

        }

        Ok(Self { elements })
    }

    pub fn elements_named<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a FormspecElement> + 'a {
        self.elements
            .iter()
            .filter(move |element| element.name == name)
    }
}

fn read_string16(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let bytes = reader.read_string16()?;
    String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidFormspec("packet string is not UTF-8"))
}

fn read_string32(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let length = reader.read_len32()?;
    if length > MAX_FORMSPEC_BYTES {
        return Err(ProtocolError::InvalidFormspec(
            "formspec exceeds size limit",
        ));
    }
    let bytes = reader.read_bytes(length)?;
    String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidFormspec("formspec is not UTF-8"))
}

pub fn decode_inventory_formspec(data: &[u8]) -> Result<InventoryFormspec, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let formspec = read_string32(&mut reader)?;
    if !reader.is_empty() {
        return Err(ProtocolError::InvalidFormspec(
            "unexpected inventory formspec data",
        ));
    }
    Ok(InventoryFormspec { formspec })
}

pub fn decode_show_formspec(data: &[u8]) -> Result<ShowFormspec, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let formspec = read_string32(&mut reader)?;
    let form_name = read_string16(&mut reader)?;
    if !reader.is_empty() {
        return Err(ProtocolError::InvalidFormspec(
            "unexpected show formspec data",
        ));
    }
    Ok(ShowFormspec {
        formspec,
        form_name,
    })
}

pub fn decode_detached_inventory(data: &[u8]) -> Result<DetachedInventoryUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let name = read_string16(&mut reader)?;
    let keep_inventory = reader.read_u8()? != 0;

    let serialized_inventory = if keep_inventory {
        let _legacy_length = reader.read_u16()?;
        reader.read_bytes(reader.remaining())?.to_vec()
    } else {
        if !reader.is_empty() {
            return Err(ProtocolError::InvalidFormspec(
                "unexpected data after detached inventory removal",
            ));
        }
        Vec::new()
    };

    Ok(DetachedInventoryUpdate {
        name,
        keep_inventory,
        serialized_inventory,
    })
}

pub fn decode_formspec_prepend(data: &[u8]) -> Result<FormspecPrepend, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let formspec = read_string16(&mut reader)?;
    if !reader.is_empty() {
        return Err(ProtocolError::InvalidFormspec(
            "unexpected formspec prepend data",
        ));
    }
    Ok(FormspecPrepend { formspec })
}

fn put_fields(writer: &mut PacketWriter, fields: &[FormspecField]) -> Result<(), ProtocolError> {
    let count = u16::try_from(fields.len()).map_err(|_| ProtocolError::InvalidStringLength)?;
    writer.write_u16(count);

    for field in fields {
        writer.write_string16(&field.name)?;
        writer.write_string32(&field.value)?;
    }

    Ok(())
}

pub fn encode_inventory_fields(
    form_name: &str,
    fields: &[FormspecField],
) -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::with_capacity(8 + form_name.len());
    writer.write_u16(u16::from(ToServerCommand::InventoryFields));
    writer.write_string16(form_name)?;
    put_fields(&mut writer, fields)?;
    Ok(writer.into_inner())
}

pub fn encode_node_metadata_fields(
    position: I16Vec3,
    form_name: &str,
    fields: &[FormspecField],
) -> Result<Bytes, ProtocolError> {
    let mut writer = PacketWriter::with_capacity(16 + form_name.len());
    writer.write_u16(u16::from(ToServerCommand::NodeMetaFields));
    writer.write_i16(position.x);
    writer.write_i16(position.y);
    writer.write_i16(position.z);
    writer.write_string16(form_name)?;
    put_fields(&mut writer, fields)?;
    Ok(writer.into_inner())
}
