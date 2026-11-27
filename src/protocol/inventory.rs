use ahash::{AHashMap, AHashSet};

use crate::errors::ProtocolError;
use crate::protocol::constant::{MAX_PROTOCOL_VERSION, MAX_SPLIT_PACKET_PAYLOAD_SIZE};
use crate::protocol::itemdef::{
    ITEM_TYPE_TOOL, ItemRegistry, ToolCapabilities, ToolGroupCapability, WearBarBlendMode,
    WearBarParams,
};
use crate::protocol::reader::PacketReader;

const MAX_INVENTORY_BYTES: usize = MAX_SPLIT_PACKET_PAYLOAD_SIZE;

const MAX_INVENTORY_SLOT_STORAGE_BYTES: usize = MAX_INVENTORY_BYTES;
const MAX_INVENTORY_SLOT_COUNT: usize = MAX_INVENTORY_SLOT_STORAGE_BYTES
    / (std::mem::size_of::<SlotUpdate>() + std::mem::size_of::<Option<ItemStack>>());
const PROTOCOL_INVENTORY_LONG_STRING: u16 = 52;
const PROTOCOL_INCREMENTAL_INVENTORY: u16 = 38;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub serialized: String,
    name: String,
    amount: u16,
    item_wear: u16,
    metadata: AHashMap<String, String>,
    tool_capabilities_override: Option<ToolCapabilities>,
    wear_bar_params_override: Option<WearBarParams>,
}

impl Eq for ItemStack {}

impl ItemStack {
    pub fn from_serialized(serialized: String) -> Result<Self, ProtocolError> {
        let tokens = itemstring_tokens(&serialized);
        let Some(kind) = tokens.first().copied() else {
            return Err(ProtocolError::InvalidInventory("empty item string"));
        };

        let (name_token, amount, item_wear, metadata_index) = match kind {
            "MaterialItem" | "MaterialItem2" => {
                let material = tokens
                    .get(1)
                    .copied()
                    .ok_or(ProtocolError::InvalidInventory(
                        "missing legacy material id",
                    ))?
                    .parse::<u16>()
                    .map_err(|_| ProtocolError::InvalidInventory("invalid legacy material id"))?;
                let amount = tokens
                    .get(2)
                    .copied()
                    .ok_or(ProtocolError::InvalidInventory(
                        "missing legacy material count",
                    ))?
                    .parse::<u16>()
                    .map_err(|_| {
                        ProtocolError::InvalidInventory("invalid legacy material count")
                    })?;
                let name = legacy_material_name(material, kind == "MaterialItem")?;
                let is_empty = amount == 0;
                return Ok(Self {
                    serialized,
                    name: if is_empty {
                        String::new()
                    } else {
                        name.to_owned()
                    },
                    amount,
                    item_wear: 0,
                    metadata: AHashMap::new(),
                    tool_capabilities_override: None,
                    wear_bar_params_override: None,
                });
            }
            "MBOItem" => {
                return Err(ProtocolError::InvalidInventory(
                    "unsupported legacy MBO item",
                ));
            }
            "node" | "NodeItem" | "MaterialItem3" | "craft" | "CraftItem" => {
                let name = tokens
                    .get(1)
                    .copied()
                    .ok_or(ProtocolError::InvalidInventory("missing legacy item name"))?;
                let count = tokens
                    .get(2)
                    .copied()
                    .ok_or(ProtocolError::InvalidInventory("missing legacy item count"))?
                    .parse::<i32>()
                    .map_err(|_| ProtocolError::InvalidInventory("invalid legacy item count"))?;
                let count = if count == 0 { 1 } else { count as u16 };
                (name, count, 0, None)
            }
            "tool" | "ToolItem" => {
                let name = tokens
                    .get(1)
                    .copied()
                    .ok_or(ProtocolError::InvalidInventory("missing legacy tool name"))?;
                let wear = tokens
                    .get(2)
                    .copied()
                    .ok_or(ProtocolError::InvalidInventory("missing legacy tool wear"))?
                    .parse::<i32>()
                    .map_err(|_| ProtocolError::InvalidInventory("invalid legacy tool wear"))?
                    as u16;
                (name, 1, wear, None)
            }
            _ => {
                let name = kind;
                let count = tokens.get(1).copied().map_or(1, |value| {
                    value.parse::<i64>().map_or(1, |count| count as u16)
                });
                let wear = tokens
                    .get(2)
                    .copied()
                    .map(str::parse::<i32>)
                    .transpose()
                    .map_err(|_| ProtocolError::InvalidInventory("invalid item wear"))?
                    .map_or(0, |wear| wear as u16);
                (name, count, wear, Some(3))
            }
        };

        let name = decode_itemstring_string(name_token).ok_or(ProtocolError::InvalidInventory(
            "invalid item name encoding",
        ))?;
        let metadata = metadata_index
            .and_then(|index| tokens.get(index).copied())
            .map(decode_itemstack_metadata)
            .transpose()?
            .unwrap_or_default();

        let is_empty = name.is_empty() || amount == 0;
        let metadata = if is_empty { AHashMap::new() } else { metadata };
        let tool_capabilities_override = parse_tool_capabilities(&metadata);
        let wear_bar_params_override = parse_wear_bar_params(&metadata);

        Ok(Self {
            serialized,
            name: if is_empty { String::new() } else { name },
            amount: if is_empty { 0 } else { amount },
            item_wear: if is_empty { 0 } else { item_wear },
            metadata,
            tool_capabilities_override,
            wear_bar_params_override,
        })
    }

    pub fn normalize_with_item_definitions(&mut self, definitions: &ItemRegistry) {
        if self.name.is_empty() {
            return;
        }

        self.name = definitions.resolve_name(&self.name).to_owned();

        if definitions
            .get_by_name(&self.name)
            .is_some_and(|definition| definition.item_type == ITEM_TYPE_TOOL)
        {
            self.amount = 1;
        }
    }

    #[must_use]
    pub fn item_name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn count(&self) -> u16 {
        self.amount
    }

    #[must_use]
    pub const fn wear(&self) -> u16 {
        self.item_wear
    }

    #[must_use]
    pub fn metadata_value(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
    }

    #[must_use]
    pub fn tool_capabilities_override(&self) -> Option<&ToolCapabilities> {
        self.tool_capabilities_override.as_ref()
    }

    #[must_use]
    pub fn wear_bar_params_override(&self) -> Option<&WearBarParams> {
        self.wear_bar_params_override.as_ref()
    }

    #[must_use]
    pub fn display_count(&self) -> Option<&str> {
        self.metadata_value("count_meta")
            .filter(|value| !value.is_empty())
    }

    #[must_use]
    pub fn count_alignment(&self) -> (u8, u8) {
        let value = self
            .metadata_value("count_alignment")
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(0);
        let horizontal = match value.rem_euclid(4) {
            1 => 1,
            2 => 2,
            _ => 3,
        };
        let vertical = match value.div_euclid(4).clamp(0, 3) {
            1 => 1,
            2 => 2,
            _ => 3,
        };
        (horizontal, vertical)
    }

    #[must_use]
    pub fn inventory_image_override(&self) -> Option<&str> {
        self.metadata_value("inventory_image")
            .filter(|value| !value.is_empty())
    }

    #[must_use]
    pub fn inventory_overlay_override(&self) -> Option<&str> {
        self.metadata_value("inventory_overlay")
            .filter(|value| !value.is_empty())
    }

    #[must_use]
    pub fn wield_image_override(&self) -> Option<&str> {
        self.metadata_value("wield_image")
            .filter(|value| !value.is_empty())
    }

    #[must_use]
    pub fn wield_overlay_override(&self) -> Option<&str> {
        self.metadata_value("wield_overlay")
            .filter(|value| !value.is_empty())
    }

    #[must_use]
    pub fn wield_scale_override(&self) -> Option<&str> {
        self.metadata_value("wield_scale")
            .filter(|value| !value.is_empty())
    }
}

fn parse_tool_capabilities(metadata: &AHashMap<String, String>) -> Option<ToolCapabilities> {
    let value =
        serde_json::from_str::<serde_json::Value>(metadata.get("tool_capabilities")?).ok()?;
    let object = value.as_object()?;

    let full_punch_interval = json_f32(object.get("full_punch_interval"))?.unwrap_or(1.4);
    let max_drop_level = json_i16(object.get("max_drop_level"))?.unwrap_or(1);
    let punch_attack_uses = json_clamped_u16(object.get("punch_attack_uses"))?.unwrap_or(0);

    let mut groupcaps = Vec::new();
    if let Some(groups) = object.get("groupcaps") {
        let groups = groups.as_object()?;
        groupcaps.reserve(groups.len());

        for (name, value) in groups {
            let fields = value.as_object()?;
            let uses = json_i16(fields.get("uses"))?.unwrap_or(20);
            let max_level = json_i16(fields.get("maxlevel"))?.unwrap_or(1);
            let mut times = Vec::new();

            if let Some(values) = fields.get("times") {
                let values = values.as_array()?;
                times.reserve(values.len());

                for (rating, time) in values.iter().enumerate() {
                    let Some(time) = time.as_f64() else {
                        continue;
                    };
                    let rating = i16::try_from(rating).ok()?;
                    let time = time as f32;
                    if time.is_finite() {
                        times.push((rating, time));
                    }
                }
            }

            groupcaps.push(ToolGroupCapability {
                name: name.clone(),
                uses,
                max_level,
                times,
            });
        }
    }

    let mut damage_groups = Vec::new();
    if let Some(groups) = object.get("damage_groups") {
        let groups = groups.as_object()?;
        damage_groups.reserve(groups.len());

        for (name, value) in groups {
            let rating = value.as_i64().and_then(|value| i16::try_from(value).ok())?;
            damage_groups.push((name.clone(), rating));
        }
    }

    Some(ToolCapabilities {
        full_punch_interval,
        max_drop_level,
        groupcaps,
        damage_groups,
        punch_attack_uses: Some(punch_attack_uses),
    })
}

fn parse_wear_bar_params(metadata: &AHashMap<String, String>) -> Option<WearBarParams> {
    let value = serde_json::from_str::<serde_json::Value>(metadata.get("wear_color")?).ok()?;
    let object = value.as_object()?;
    let blend = match object.get("blend")?.as_str()? {
        "constant" => WearBarBlendMode::Constant,
        "linear" => WearBarBlendMode::Linear,
        _ => return None,
    };
    let stops = object.get("color_stops")?.as_object()?;
    let mut color_stops = Vec::with_capacity(stops.len());

    for (position, value) in stops {
        let position = position.parse::<f32>().ok()?;
        if !position.is_finite() || !(0.0..=1.0).contains(&position) {
            return None;
        }
        let color = parse_wear_color(value.as_str()?)?;
        color_stops.push((position, color));
    }

    color_stops.sort_unstable_by(|left, right| left.0.total_cmp(&right.0));
    (!color_stops.is_empty()).then_some(WearBarParams { color_stops, blend })
}

fn json_f32(value: Option<&serde_json::Value>) -> Option<Option<f32>> {
    value.map_or(Some(None), |value| {
        let number = value.as_f64()?;
        let number = number as f32;
        number.is_finite().then_some(Some(number))
    })
}

fn json_i16(value: Option<&serde_json::Value>) -> Option<Option<i16>> {
    value.map_or(Some(None), |value| {
        value
            .as_i64()
            .and_then(|number| i16::try_from(number).ok())
            .map(Some)
    })
}

fn json_clamped_u16(value: Option<&serde_json::Value>) -> Option<Option<u16>> {
    value.map_or(Some(None), |value| {
        value
            .as_i64()
            .and_then(|number| i32::try_from(number).ok())
            .and_then(|number| u16::try_from(number.clamp(0, i32::from(u16::MAX))).ok())
            .map(Some)
    })
}

fn parse_wear_color(value: &str) -> Option<[u8; 4]> {
    let color = if let Ok(color) = csscolorparser::parse(value) {
        color
    } else {
        let (name, alpha) = value.split_once('#')?;
        if name.is_empty() || !(1..=2).contains(&alpha.len()) {
            return None;
        }

        let mut color = csscolorparser::parse(name).ok()?.to_rgba8();
        let alpha = u8::from_str_radix(alpha, 16).ok()?;
        color[3] = if alpha <= 0x0f { alpha * 0x11 } else { alpha };
        return Some(color);
    };

    Some(color.to_rgba8())
}

fn legacy_material_name(
    material: u16,
    translate_version_19: bool,
) -> Result<&'static str, ProtocolError> {
    let material = if translate_version_19 && material <= 0xff {
        match material {
            1 => 0x800,
            4 => 0x801,
            5 => 0x802,
            6 => 0x803,
            7 => 0x804,
            8 => 0x805,
            10 => 0x806,
            11 => 0x807,
            12 => 0x808,
            13 => 0x809,
            18 => 0x80a,
            19 => 0x80b,
            20 => 0x80c,
            22 => 0x80d,
            23 => 0x80e,
            24 => 0x80f,
            25 => 0x810,
            26 => 0x811,
            27 => 0x812,
            28 => 0x813,
            29 => 0x814,
            value => value,
        }
    } else {
        material
    };

    if material > 0x0fff {
        return Err(ProtocolError::InvalidInventory(
            "legacy material id exceeds range",
        ));
    }

    Ok(match material {
        0 => "default:stone",
        2 => "default:water_flowing",
        3 => "default:torch",
        9 => "default:water_source",
        14 => "default:sign_wall",
        15 => "default:chest",
        16 => "default:furnace",
        17 => "default:chest_locked",
        21 => "default:fence_wood",
        30 => "default:rail",
        31 => "default:ladder",
        32 => "default:lava_flowing",
        33 => "default:lava_source",
        0x7e => "air",
        0x7f => "ignore",
        0x800 => "default:dirt_with_grass",
        0x801 => "default:tree",
        0x802 => "default:leaves",
        0x803 => "default:dirt_with_grass_footsteps",
        0x804 => "default:mese",
        0x805 => "default:dirt",
        0x806 => "default:cloud",
        0x807 => "default:coalstone",
        0x808 => "default:wood",
        0x809 => "default:sand",
        0x80a => "default:cobble",
        0x80b => "default:steelblock",
        0x80c => "default:glass",
        0x80d => "default:mossycobble",
        0x80e => "default:gravel",
        0x80f => "default:sandstone",
        0x810 => "default:cactus",
        0x811 => "default:brick",
        0x812 => "default:clay",
        0x813 => "default:papyrus",
        0x814 => "default:bookshelf",
        0x815 => "default:jungletree",
        0x816 => "default:junglegrass",
        0x817 => "default:nyancat",
        0x818 => "default:nyancat_rainbow",
        0x819 => "default:apple",
        0x820 => "default:sapling",
        _ => "unknown_block",
    })
}

fn decode_itemstring_string(token: &str) -> Option<String> {
    let bytes = token.as_bytes();
    if bytes.first() != Some(&b'"') {
        return Some(token.to_owned());
    }
    if bytes.last() != Some(&b'"') || bytes.len() < 2 {
        return None;
    }

    let mut decoded = Vec::with_capacity(bytes.len() - 2);
    let mut index = 1;
    while index < bytes.len() - 1 {
        let byte = bytes[index];
        index += 1;
        if byte != b'\\' {
            decoded.push(byte);
            continue;
        }

        let escaped = *bytes.get(index)?;
        index += 1;
        match escaped {
            b'b' => decoded.push(b'\x08'),
            b'f' => decoded.push(b'\x0c'),
            b'n' => decoded.push(b'\n'),
            b'r' => decoded.push(b'\r'),
            b't' => decoded.push(b'\t'),
            b'u' => {
                let digits = bytes.get(index..index.checked_add(4)?)?;
                let mut value = 0_u16;
                for digit in digits {
                    let nibble = match digit {
                        b'0'..=b'9' => digit - b'0',
                        b'a'..=b'f' => digit - b'a' + 10,
                        b'A'..=b'F' => digit - b'A' + 10,
                        _ => return None,
                    };
                    value = (value << 4) | u16::from(nibble);
                }
                decoded.push(value.to_le_bytes()[0]);
                index += 4;
            }
            other => decoded.push(other),
        }
    }

    String::from_utf8(decoded).ok()
}

fn decode_itemstack_metadata(token: &str) -> Result<AHashMap<String, String>, ProtocolError> {
    let decoded = decode_itemstring_string(token).ok_or(ProtocolError::InvalidInventory(
        "invalid item metadata encoding",
    ))?;
    let Some(body) = decoded.strip_prefix('\u{1}') else {
        let mut metadata = AHashMap::new();
        if !decoded.is_empty() {
            metadata.insert(String::new(), decoded);
        }
        return Ok(metadata);
    };

    let mut metadata = AHashMap::with_capacity(8);

    for entry in body.split_terminator('\u{3}') {
        let (key, value) = entry
            .split_once('\u{2}')
            .map_or((entry, ""), |(key, value)| (key, value));
        metadata.insert(key.to_owned(), value.to_owned());
    }

    Ok(metadata)
}

fn itemstring_tokens(serialized: &str) -> Vec<&str> {
    let mut tokens = Vec::with_capacity(5);
    let mut start = None;
    let mut quoted = false;
    let mut escaped = false;

    for (index, character) in serialized.char_indices() {
        if start.is_none() {
            if character.is_whitespace() {
                continue;
            }
            start = Some(index);
            quoted = character == '"';
            continue;
        }

        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
        } else if character.is_whitespace()
            && let Some(token_start) = start.take()
        {
            tokens.push(&serialized[token_start..index]);
        }
    }

    if let Some(token_start) = start {
        tokens.push(&serialized[token_start..]);
    }

    tokens
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InventoryList {
    pub width: u32,
    pub slots: Vec<Option<ItemStack>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Inventory {
    pub lists: AHashMap<String, InventoryList>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SlotUpdate {
    Keep,
    Empty,
    Item(ItemStack),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ListUpdate {
    KeepList {
        name: String,
    },
    Replace {
        name: String,
        width: u32,
        slots: Vec<SlotUpdate>,
        size: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryUpdate {
    operations: Vec<ListUpdate>,
    pub skip_wield_anim: bool,
}

impl Inventory {
    pub fn from_serialized(text: &str) -> Result<Self, ProtocolError> {
        let mut inventory = Self::default();
        inventory.apply_serialized(text)?;
        Ok(inventory)
    }

    pub fn apply_serialized(&mut self, text: &str) -> Result<(), ProtocolError> {
        if text.len() > MAX_INVENTORY_BYTES {
            return Err(ProtocolError::PacketTooLarge);
        }

        let operations = parse_inventory_text(text, MAX_PROTOCOL_VERSION)?;
        self.apply_update(InventoryUpdate {
            operations,
            skip_wield_anim: true,
        });
        Ok(())
    }

    pub fn apply_update(&mut self, update: InventoryUpdate) {
        let mut retained = AHashSet::with_capacity(update.operations.len());

        for operation in update.operations {
            match operation {
                ListUpdate::KeepList { name } => {
                    if self.lists.contains_key(&name) {
                        retained.insert(name);
                    }
                }
                ListUpdate::Replace {
                    name,
                    width,
                    slots,
                    size,
                } => {
                    let previous = self.lists.get(&name);
                    let mut new_slots = Vec::with_capacity(size);

                    for (index, slot) in slots.into_iter().enumerate() {
                        let value = match slot {
                            SlotUpdate::Keep => previous
                                .and_then(|list| list.slots.get(index))
                                .cloned()
                                .flatten(),
                            SlotUpdate::Empty => None,
                            SlotUpdate::Item(item) => Some(item),
                        };

                        new_slots.push(value);
                    }

                    new_slots.resize(size, None);
                    retained.insert(name.clone());
                    self.lists.insert(
                        name,
                        InventoryList {
                            width,
                            slots: new_slots,
                        },
                    );
                }
            }
        }

        self.lists.retain(|name, _| retained.contains(name));
    }

    pub fn normalize_item_stacks(&mut self, definitions: &ItemRegistry) {
        for stack in self
            .lists
            .values_mut()
            .flat_map(|list| list.slots.iter_mut().flatten())
        {
            stack.normalize_with_item_definitions(definitions);
        }
    }

    #[must_use]
    pub fn get_list(&self, name: &str) -> Option<&InventoryList> {
        self.lists.get(name)
    }

    #[must_use]
    pub fn list_count(&self) -> usize {
        self.lists.len()
    }
}

pub fn decode_inventory(
    data: &[u8],
    protocol_version: u16,
) -> Result<InventoryUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let (inventory_bytes, skip_wield_anim) = if protocol_version >= PROTOCOL_INVENTORY_LONG_STRING {
        let inventory_length =
            usize::try_from(reader.read_u32()?).map_err(|_| ProtocolError::InvalidStringLength)?;

        if inventory_length > MAX_INVENTORY_BYTES {
            return Err(ProtocolError::PacketTooLarge);
        }

        let inventory_bytes = reader.read_bytes(inventory_length)?;

        let skip_wield_anim = match reader.read_u8()? {
            0 => false,
            1 => true,
            _ => {
                return Err(ProtocolError::InvalidInventory("invalid boolean"));
            }
        };

        (inventory_bytes, skip_wield_anim)
    } else {
        let remaining = reader.remaining();

        if remaining > MAX_INVENTORY_BYTES {
            return Err(ProtocolError::PacketTooLarge);
        }

        (reader.read_bytes(remaining)?, false)
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidInventory("trailing packet data"));
    }

    let inventory_text = std::str::from_utf8(inventory_bytes)
        .map_err(|_| ProtocolError::InvalidInventory("inventory is not UTF-8"))?;

    let operations = parse_inventory_text(inventory_text, protocol_version)?;

    Ok(InventoryUpdate {
        operations,
        skip_wield_anim,
    })
}

fn parse_inventory_text(
    text: &str,
    protocol_version: u16,
) -> Result<Vec<ListUpdate>, ProtocolError> {
    let mut operations = Vec::new();
    let mut current: Option<(String, usize, u32, Vec<SlotUpdate>)> = None;
    let mut remaining_slot_capacity = MAX_INVENTORY_SLOT_COUNT;
    let mut ended = false;

    for raw_line in text.lines() {
        let line = raw_line.trim_end_matches('\r');

        if line.is_empty() {
            continue;
        }

        if ended {
            return Err(ProtocolError::InvalidInventory("data after EndInventory"));
        }

        if let Some(rest) = line.strip_prefix("KeepList ") {
            if protocol_version < PROTOCOL_INCREMENTAL_INVENTORY {
                return Err(ProtocolError::InvalidInventory(
                    "KeepList requires protocol version 38",
                ));
            }

            let mut parts = rest.split_whitespace();
            let name = parts
                .next()
                .ok_or(ProtocolError::InvalidInventory("missing KeepList name"))?;

            if current.is_some() || parts.next().is_some() {
                return Err(ProtocolError::InvalidInventory("invalid KeepList"));
            }

            operations.push(ListUpdate::KeepList {
                name: name.to_owned(),
            });
            continue;
        }

        if let Some(rest) = line.strip_prefix("List ") {
            if current.is_some() {
                return Err(ProtocolError::InvalidInventory("nested inventory list"));
            }

            let mut parts = rest.split_whitespace();

            let name = parts
                .next()
                .ok_or(ProtocolError::InvalidInventory("missing list name"))?;

            let size = parts
                .next()
                .ok_or(ProtocolError::InvalidInventory("missing list size"))?
                .parse::<u32>()
                .map_err(|_| ProtocolError::InvalidInventory("invalid list size"))?;
            let size = usize::try_from(size)
                .map_err(|_| ProtocolError::InvalidInventory("invalid list size"))?;

            if parts.next().is_some() || size > remaining_slot_capacity {
                return Err(ProtocolError::InvalidInventory(
                    "inventory slot storage exceeds limit",
                ));
            }
            remaining_slot_capacity -= size;

            current = Some((name.to_owned(), size, 0, Vec::with_capacity(size)));
            continue;
        }

        if line == "EndInventoryList" || (line == "end" && current.is_some()) {
            let (name, size, width, mut slots) = current.take().ok_or(
                ProtocolError::InvalidInventory("list end marker without list"),
            )?;
            if slots.len() > size {
                return Err(ProtocolError::InvalidInventory("too many inventory slots"));
            }

            slots.resize(size, SlotUpdate::Empty);

            operations.push(ListUpdate::Replace {
                name,
                width,
                slots,
                size,
            });
            continue;
        }

        if line == "EndInventory" || line == "end" {
            if current.is_some() {
                return Err(ProtocolError::InvalidInventory("list not terminated"));
            }

            ended = true;
            continue;
        }

        let Some((_, size, width, slots)) = current.as_mut() else {
            if protocol_version >= PROTOCOL_INCREMENTAL_INVENTORY {
                continue;
            }
            return Err(ProtocolError::InvalidInventory("entry outside list"));
        };

        if let Some(value) = line.strip_prefix("Width ") {
            *width = value
                .parse::<u32>()
                .map_err(|_| ProtocolError::InvalidInventory("invalid list width"))?;
            continue;
        }

        if line == "Empty" {
            slots.push(SlotUpdate::Empty);
        } else if line == "Keep" {
            if protocol_version < PROTOCOL_INCREMENTAL_INVENTORY {
                return Err(ProtocolError::InvalidInventory(
                    "Keep requires protocol version 38",
                ));
            }
            slots.push(SlotUpdate::Keep);
        } else if let Some(value) = line.strip_prefix("Item ") {
            if value.is_empty() {
                return Err(ProtocolError::InvalidInventory("empty item stack"));
            }

            slots.push(SlotUpdate::Item(ItemStack::from_serialized(
                value.to_owned(),
            )?));
        } else if protocol_version < PROTOCOL_INCREMENTAL_INVENTORY {
            return Err(ProtocolError::InvalidInventory("unknown inventory entry"));
        }

        if slots.len() > *size {
            return Err(ProtocolError::InvalidInventory("too many inventory slots"));
        }
    }

    if current.is_some() || !ended {
        return Err(ProtocolError::InvalidInventory("missing EndInventory"));
    }

    Ok(operations)
}
