use crate::errors::ProtocolError;
use crate::protocol::constant::MAX_SPLIT_PACKET_PAYLOAD_SIZE;
use crate::protocol::reader::PacketReader;
use glam::{Vec2, Vec3};
const MAX_ACTIVE_OBJECT_INIT_DATA_SIZE: usize = MAX_SPLIT_PACKET_PAYLOAD_SIZE;
pub const ACTIVE_OBJECT_TYPE_LUAENTITY: u8 = 7;
pub const ACTIVE_OBJECT_TYPE_PLAYER: u8 = 100;
pub const ACTIVE_OBJECT_TYPE_GENERIC: u8 = 101;
pub const ACTIVE_OBJECT_STOP_ANIMATION_PROTOCOL_VERSION: u16 = 52;
pub const ACTIVE_OBJECT_ANIMATION_TRACK_PROTOCOL_VERSION: u16 = 53;
pub const BONE_POSITION_ABSOLUTE_FLAG: u8 = 1 << 0;
pub const BONE_ROTATION_ABSOLUTE_FLAG: u8 = 1 << 1;
pub const BONE_SCALE_ABSOLUTE_FLAG: u8 = 1 << 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenericActiveObjectTrackId {
    Number(u16),
    Name(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericActiveObjectAnimation {
    pub frame_range: Vec2,
    pub fps: f32,
    pub blend_duration: f32,
    pub looping: bool,
    pub track: GenericActiveObjectTrackId,
    pub priority: Option<i32>,
    pub current_frame: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericActiveObjectArmorGroup {
    pub name: String,
    pub rating: i16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericActiveObjectBonePosition {
    pub bone: String,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Option<Vec3>,
    pub interpolation: Option<[f32; 3]>,
    pub transform_flags: Option<u8>,
}

impl GenericActiveObjectBonePosition {
    #[must_use]
    pub fn position_is_absolute(&self) -> bool {
        self.transform_flags
            .is_none_or(|flags| flags & BONE_POSITION_ABSOLUTE_FLAG != 0)
    }

    #[must_use]
    pub fn rotation_is_absolute(&self) -> bool {
        self.transform_flags
            .is_none_or(|flags| flags & BONE_ROTATION_ABSOLUTE_FLAG != 0)
    }

    #[must_use]
    pub fn scale_is_absolute(&self) -> bool {
        self.transform_flags
            .is_some_and(|flags| flags & BONE_SCALE_ABSOLUTE_FLAG != 0)
    }

    #[must_use]
    pub fn interpolation_durations(&self) -> [f32; 3] {
        self.interpolation.unwrap_or([0.0; 3])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericActiveObjectAttachment {
    pub parent_id: u16,
    pub bone: String,
    pub position: Vec3,
    pub rotation: Vec3,
    pub visible_on_attach: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericActiveObjectPhysicsOverride {
    pub speed: f32,
    pub jump: f32,
    pub gravity: f32,
    pub sneak: bool,
    pub sneak_glitch: bool,
    pub new_move: bool,
    pub speed_climb: Option<f32>,
    pub speed_crouch: Option<f32>,
    pub liquid_fluidity: Option<f32>,
    pub liquid_fluidity_smooth: Option<f32>,
    pub liquid_sink: Option<f32>,
    pub acceleration_default: Option<f32>,
    pub acceleration_air: Option<f32>,
    pub speed_fast: Option<f32>,
    pub acceleration_fast: Option<f32>,
    pub speed_walk: Option<f32>,
    pub extensions: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenericActiveObjectInfant {
    pub id: u16,
    pub object_type: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericActiveObjectPositionUpdate {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub rotation: Vec3,
    pub interpolate: bool,
    pub is_end: bool,
    pub interval: f32,
    pub interpolate_rotation: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericActiveObjectState {
    pub name: String,
    pub is_player: bool,
    pub hp: u16,
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub rotation: Vec3,
    pub properties: Option<GenericObjectProperties>,
    pub interpolate: bool,
    pub is_end: bool,
    pub interval: f32,
    pub interpolate_rotation: bool,
    pub texture_modifier: String,
    pub sprite: Option<GenericActiveObjectSprite>,
    pub armor_groups: Vec<GenericActiveObjectArmorGroup>,
    pub bone_positions: Vec<GenericActiveObjectBonePosition>,
    pub attachment: Option<GenericActiveObjectAttachment>,
    pub physics_override: Option<GenericActiveObjectPhysicsOverride>,
    pub infants: Vec<GenericActiveObjectInfant>,
    pub animations: Vec<GenericActiveObjectAnimation>,
    pub nametag_color: Option<[u8; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericActiveObjectSprite {
    pub base_position: [i16; 2],
    pub frame_count: u16,
    pub frame_length: f32,
    pub select_horiz_by_yaw_pitch: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericObjectProperties {
    pub version: u8,
    pub hp_max: u16,
    pub physical: bool,
    pub collisionbox_min: Vec3,
    pub collisionbox_max: Vec3,
    pub selectionbox_min: Vec3,
    pub selectionbox_max: Vec3,
    pub pointable: u8,
    pub visual: String,
    pub visual_size: Vec3,
    pub textures: Vec<String>,
    pub spritediv: [i16; 2],
    pub initial_sprite_basepos: [i16; 2],
    pub is_visible: bool,
    pub makes_footstep_sound: bool,
    pub automatic_rotate: f32,
    pub mesh: String,
    pub colors: Vec<[u8; 4]>,
    pub collide_with_objects: bool,
    pub stepheight: f32,
    pub automatic_face_movement_dir: u8,
    pub automatic_face_movement_dir_offset: f32,
    pub backface_culling: bool,
    pub nametag: String,
    pub nametag_color: [u8; 4],
    pub automatic_face_movement_max_rotation_per_sec: f32,
    pub infotext: String,
    pub wield_item: String,
    pub glow: i8,
    pub breath_max: u16,
    pub eye_height: f32,
    pub zoom_fov: f32,
    pub use_texture_alpha: u8,
    pub damage_texture_modifier: Option<String>,
    pub shaded: Option<bool>,
    pub show_on_minimap: Option<bool>,
    pub nametag_bgcolor: Option<Option<[u8; 4]>>,
    pub rotate_selectionbox: Option<bool>,
    pub node_content: Option<u16>,
    pub node_param1: Option<u8>,
    pub node_param2: Option<u8>,
    pub nametag_fontsize: Option<Option<u32>>,
    pub nametag_scale_z: Option<u8>,
    pub step_up_mode: Option<u8>,
    pub extensions: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveObjectData {
    pub id: u16,
    pub object_type: u8,
    pub initialization_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveObjectUpdate {
    pub removed: Vec<u16>,
    pub added: Vec<ActiveObjectData>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveObjectMessage {
    pub id: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericActiveObjectMessage {
    pub command: u8,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericActiveObjectInit {
    pub version: u8,
    pub name: String,
    pub is_player: bool,
    pub id: u16,
    pub position: Vec3,
    pub rotation: Vec3,
    pub hp: u16,
    pub messages: Vec<GenericActiveObjectMessage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericActiveObjectCommandKind {
    SetProperties,
    UpdatePosition,
    SetTextureModifier,
    SetSprite,
    Punched,
    UpdateArmorGroups,
    SetBonePosition,
    AttachTo,
    SetPhysicsOverride,
    SpawnInfant,
    SetAnimation,
    SetAnimationSpeed,
    StopAnimation,
    Legacy,
    Unknown(u8),
}

impl GenericActiveObjectMessage {
    #[must_use]
    pub const fn kind(&self) -> GenericActiveObjectCommandKind {
        match self.command {
            0 => GenericActiveObjectCommandKind::SetProperties,
            1 => GenericActiveObjectCommandKind::UpdatePosition,
            2 => GenericActiveObjectCommandKind::SetTextureModifier,
            3 => GenericActiveObjectCommandKind::SetSprite,
            4 => GenericActiveObjectCommandKind::Punched,
            5 => GenericActiveObjectCommandKind::UpdateArmorGroups,
            7 => GenericActiveObjectCommandKind::SetBonePosition,
            8 => GenericActiveObjectCommandKind::AttachTo,
            9 => GenericActiveObjectCommandKind::SetPhysicsOverride,
            11 => GenericActiveObjectCommandKind::SpawnInfant,
            6 => GenericActiveObjectCommandKind::SetAnimation,
            12 => GenericActiveObjectCommandKind::SetAnimationSpeed,
            13 => GenericActiveObjectCommandKind::StopAnimation,
            10 => GenericActiveObjectCommandKind::Legacy,
            other => GenericActiveObjectCommandKind::Unknown(other),
        }
    }
}

impl GenericActiveObjectState {
    pub fn from_init_data(data: &[u8], protocol_version: u16) -> Result<Self, ProtocolError> {
        let init = decode_generic_active_object_init(data)?;

        let mut state = Self {
            name: init.name,
            is_player: init.is_player,
            hp: init.hp,
            position: init.position,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            rotation: init.rotation,
            properties: None,
            interpolate: false,
            is_end: true,
            interval: 0.0,
            interpolate_rotation: true,
            texture_modifier: String::new(),
            sprite: None,
            armor_groups: Vec::new(),
            bone_positions: Vec::new(),
            attachment: None,
            physics_override: None,
            infants: Vec::new(),
            animations: Vec::new(),
            nametag_color: None,
        };

        for message in &init.messages {
            state.apply_message(message, protocol_version)?;
        }

        Ok(state)
    }

    pub fn apply_message(
        &mut self,
        message: &GenericActiveObjectMessage,
        protocol_version: u16,
    ) -> Result<bool, ProtocolError> {
        match message.kind() {
            GenericActiveObjectCommandKind::SetProperties => {
                let properties = decode_generic_object_properties(&message.payload)?;

                self.nametag_color = Some(properties.nametag_color);
                self.properties = Some(properties);

                Ok(true)
            }
            GenericActiveObjectCommandKind::UpdatePosition => {
                let update = decode_generic_active_object_position_update(&message.payload)?;

                self.position = update.position;
                self.velocity = update.velocity;
                self.acceleration = update.acceleration;
                self.rotation = update.rotation;
                self.interpolate = update.interpolate;
                self.is_end = update.is_end;
                self.interval = update.interval;
                self.interpolate_rotation = update.interpolate_rotation;

                Ok(true)
            }
            GenericActiveObjectCommandKind::SetTextureModifier => {
                self.texture_modifier =
                    decode_generic_active_object_texture_modifier(&message.payload)?;

                Ok(true)
            }
            GenericActiveObjectCommandKind::SetSprite => {
                self.sprite = Some(decode_generic_active_object_sprite(&message.payload)?);

                Ok(true)
            }
            GenericActiveObjectCommandKind::Punched => {
                self.hp = decode_generic_active_object_hp_update(&message.payload)?;

                Ok(true)
            }
            GenericActiveObjectCommandKind::UpdateArmorGroups => {
                self.armor_groups = decode_generic_active_object_armor_groups(&message.payload)?;

                Ok(true)
            }
            GenericActiveObjectCommandKind::SetBonePosition => {
                let update = decode_generic_active_object_bone_position(&message.payload)?;

                if let Some(existing) = self
                    .bone_positions
                    .iter_mut()
                    .find(|entry| entry.bone == update.bone)
                {
                    *existing = update;
                } else {
                    self.bone_positions.push(update);
                }

                Ok(true)
            }
            GenericActiveObjectCommandKind::AttachTo => {
                let attachment = decode_generic_active_object_attachment(&message.payload)?;

                self.attachment = if attachment.parent_id == 0 {
                    None
                } else {
                    Some(attachment)
                };

                Ok(true)
            }
            GenericActiveObjectCommandKind::SetPhysicsOverride => {
                self.physics_override = Some(decode_generic_active_object_physics_override(
                    &message.payload,
                )?);

                Ok(true)
            }
            GenericActiveObjectCommandKind::SpawnInfant => {
                let infant =
                    decode_generic_active_object_infant(&message.payload, protocol_version)?;

                self.infants.push(infant);

                Ok(true)
            }
            GenericActiveObjectCommandKind::SetAnimation => {
                let animation =
                    decode_generic_active_object_animation(&message.payload, protocol_version)?;

                if let Some(existing) = self
                    .animations
                    .iter_mut()
                    .find(|entry| entry.track == animation.track)
                {
                    *existing = animation;
                } else {
                    self.animations.push(animation);
                }

                Ok(true)
            }
            GenericActiveObjectCommandKind::SetAnimationSpeed => {
                let (fps, track) = decode_generic_active_object_animation_speed(
                    &message.payload,
                    protocol_version,
                )?;

                if let Some(animation) = self
                    .animations
                    .iter_mut()
                    .find(|entry| entry.track == track)
                {
                    animation.fps = fps;
                }

                Ok(true)
            }
            GenericActiveObjectCommandKind::StopAnimation => {
                let track = decode_generic_active_object_stop_animation(
                    &message.payload,
                    protocol_version,
                )?;

                self.animations.retain(|animation| animation.track != track);

                Ok(true)
            }
            GenericActiveObjectCommandKind::Legacy => Ok(true),
            GenericActiveObjectCommandKind::Unknown(_) => Ok(false),
        }
    }
}

pub fn decode_generic_active_object_armor_groups(
    data: &[u8],
) -> Result<Vec<GenericActiveObjectArmorGroup>, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let count = usize::from(reader.read_u16()?);

    if count > reader.remaining() / 4 {
        return Err(ProtocolError::InvalidActiveObject(
            "invalid armor group count",
        ));
    }

    let mut groups = Vec::with_capacity(count);

    for _ in 0..count {
        let bytes = reader.read_string16()?;
        let name = String::from_utf8(bytes.to_vec())
            .map_err(|_| ProtocolError::InvalidActiveObject("armor group name is not UTF-8"))?;

        let rating = reader.read_i16()?;

        groups.push(GenericActiveObjectArmorGroup { name, rating });
    }

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after armor groups",
        ));
    }

    Ok(groups)
}

fn read_generic_active_object_track_id(
    reader: &mut PacketReader<'_>,
) -> Result<GenericActiveObjectTrackId, ProtocolError> {
    let number = reader.read_u16()?;

    if number != 0 {
        return Ok(GenericActiveObjectTrackId::Number(number - 1));
    }

    let bytes = reader.read_string16()?;
    let name = String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidActiveObject("animation track name is not UTF-8"))?;

    Ok(GenericActiveObjectTrackId::Name(name))
}

pub fn decode_generic_active_object_animation(
    data: &[u8],
    _protocol_version: u16,
) -> Result<GenericActiveObjectAnimation, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let frame_range = Vec2::new(reader.read_f32()?, reader.read_f32()?);
    let fps = reader.read_f32()?;
    let blend_duration = reader.read_f32()?;
    let looping = reader.read_u8()? == 0;

    let (track, priority, current_frame) = if reader.is_empty() {
        (GenericActiveObjectTrackId::Number(0), None, None)
    } else {
        let track = read_generic_active_object_track_id(&mut reader)?;
        let priority = reader.read_i32()?;
        let current_frame = reader.read_f32()?.max(0.0);

        (track, Some(priority), Some(current_frame))
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after animation update",
        ));
    }

    Ok(GenericActiveObjectAnimation {
        frame_range,
        fps,
        blend_duration,
        looping,
        track,
        priority,
        current_frame,
    })
}

pub fn decode_generic_active_object_animation_speed(
    data: &[u8],
    _protocol_version: u16,
) -> Result<(f32, GenericActiveObjectTrackId), ProtocolError> {
    let mut reader = PacketReader::new(data);

    let fps = reader.read_f32()?;

    let track = if reader.is_empty() {
        GenericActiveObjectTrackId::Number(0)
    } else {
        read_generic_active_object_track_id(&mut reader)?
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after animation speed",
        ));
    }

    Ok((fps, track))
}

pub fn decode_generic_active_object_stop_animation(
    data: &[u8],
    protocol_version: u16,
) -> Result<GenericActiveObjectTrackId, ProtocolError> {
    if protocol_version < ACTIVE_OBJECT_STOP_ANIMATION_PROTOCOL_VERSION {
        return Err(ProtocolError::InvalidActiveObject(
            "stop animation requires protocol version 52",
        ));
    }

    let mut reader = PacketReader::new(data);
    let track = read_generic_active_object_track_id(&mut reader)?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after stop animation",
        ));
    }

    Ok(track)
}

pub fn decode_generic_active_object_bone_position(
    data: &[u8],
) -> Result<GenericActiveObjectBonePosition, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let bytes = reader.read_string16()?;
    let bone = String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidActiveObject("bone name is not UTF-8"))?;

    let position = read_init_vec3(&mut reader)?;
    let rotation = read_init_vec3(&mut reader)?;

    let (scale, interpolation, transform_flags) = match reader.remaining() {
        0 => (None, None, None),
        25 => {
            let scale = read_init_vec3(&mut reader)?;
            let interpolation = [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?];
            let transform_flags = reader.read_u8()?;

            (Some(scale), Some(interpolation), Some(transform_flags))
        }
        _ => {
            return Err(ProtocolError::InvalidActiveObject(
                "invalid bone position extension length",
            ));
        }
    };

    Ok(GenericActiveObjectBonePosition {
        bone,
        position,
        rotation,
        scale,
        interpolation,
        transform_flags,
    })
}

pub fn decode_generic_active_object_attachment(
    data: &[u8],
) -> Result<GenericActiveObjectAttachment, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let parent_id = reader.read_u16()?;

    let bytes = reader.read_string16()?;
    let bone = String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidActiveObject("attachment bone name is not UTF-8"))?;

    let position = read_init_vec3(&mut reader)?;
    let rotation = read_init_vec3(&mut reader)?;
    let visible_on_attach = if reader.is_empty() {
        false
    } else {
        reader.read_u8()? != 0
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after attachment",
        ));
    }

    Ok(GenericActiveObjectAttachment {
        parent_id,
        bone,
        position,
        rotation,
        visible_on_attach,
    })
}

pub fn decode_generic_active_object_physics_override(
    data: &[u8],
) -> Result<GenericActiveObjectPhysicsOverride, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let speed = reader.read_f32()?;
    let jump = reader.read_f32()?;
    let gravity = reader.read_f32()?;
    let sneak = reader.read_u8()? == 0;
    let sneak_glitch = reader.read_u8()? == 0;
    let new_move = reader.read_u8()? == 0;

    let (
        speed_climb,
        speed_crouch,
        liquid_fluidity,
        liquid_fluidity_smooth,
        liquid_sink,
        acceleration_default,
        acceleration_air,
    ) = if reader.is_empty() {
        (None, None, None, None, None, None, None)
    } else {
        (
            Some(reader.read_f32()?),
            Some(reader.read_f32()?),
            Some(reader.read_f32()?),
            Some(reader.read_f32()?),
            Some(reader.read_f32()?),
            Some(reader.read_f32()?),
            Some(reader.read_f32()?),
        )
    };

    let (speed_fast, acceleration_fast, speed_walk) = if reader.is_empty() {
        (None, None, None)
    } else {
        (
            Some(reader.read_f32()?),
            Some(reader.read_f32()?),
            Some(reader.read_f32()?),
        )
    };

    let extensions = reader.read_bytes(reader.remaining())?.to_vec();

    Ok(GenericActiveObjectPhysicsOverride {
        speed,
        jump,
        gravity,
        sneak,
        sneak_glitch,
        new_move,
        speed_climb,
        speed_crouch,
        liquid_fluidity,
        liquid_fluidity_smooth,
        liquid_sink,
        acceleration_default,
        acceleration_air,
        speed_fast,
        acceleration_fast,
        speed_walk,
        extensions,
    })
}

pub fn decode_generic_active_object_infant(
    data: &[u8],
    protocol_version: u16,
) -> Result<GenericActiveObjectInfant, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let id = reader.read_u16()?;
    let object_type = reader.read_u8()?;

    if protocol_version < 38 {
        let initialization_data_length = reader.read_len32()?;

        if initialization_data_length > MAX_ACTIVE_OBJECT_INIT_DATA_SIZE {
            return Err(ProtocolError::InvalidActiveObject(
                "spawned infant initialization data exceeds size limit",
            ));
        }

        let _initialization_data = reader.read_bytes(initialization_data_length)?;
    }

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after spawned infant",
        ));
    }

    Ok(GenericActiveObjectInfant { id, object_type })
}

pub fn decode_generic_active_object_texture_modifier(data: &[u8]) -> Result<String, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let bytes = reader.read_string16()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after texture modifier",
        ));
    }

    String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidActiveObject("texture modifier is not UTF-8"))
}

pub fn decode_generic_active_object_sprite(
    data: &[u8],
) -> Result<GenericActiveObjectSprite, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let base_position = [reader.read_i16()?, reader.read_i16()?];
    let frame_count = reader.read_u16()?;
    let frame_length = reader.read_f32()?;
    let select_horiz_by_yaw_pitch = reader.read_u8()? != 0;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after sprite update",
        ));
    }

    Ok(GenericActiveObjectSprite {
        base_position,
        frame_count,
        frame_length,
        select_horiz_by_yaw_pitch,
    })
}

pub fn decode_generic_active_object_nametag_attributes(
    data: &[u8],
) -> Result<[u8; 4], ProtocolError> {
    let mut reader = PacketReader::new(data);
    let bytes = reader.read_bytes(4)?;

    let color = [bytes[0], bytes[1], bytes[2], bytes[3]];

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after nametag attributes",
        ));
    }

    Ok(color)
}

pub fn decode_generic_active_object_hp_update(data: &[u8]) -> Result<u16, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let hp = reader.read_u16()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after HP update",
        ));
    }

    Ok(hp)
}

pub fn decode_generic_active_object_position_update(
    data: &[u8],
) -> Result<GenericActiveObjectPositionUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let position = read_init_vec3(&mut reader)?;
    let velocity = read_init_vec3(&mut reader)?;
    let acceleration = read_init_vec3(&mut reader)?;
    let rotation = read_init_vec3(&mut reader)?;
    let interpolate = reader.read_u8()? != 0;
    let is_end = reader.read_u8()? != 0;
    let interval = reader.read_f32()?;
    let interpolate_rotation = if reader.is_empty() {
        true
    } else {
        reader.read_u8()? != 0
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after position update",
        ));
    }

    Ok(GenericActiveObjectPositionUpdate {
        position,
        velocity,
        acceleration,
        rotation,
        interpolate,
        is_end,
        interval,
        interpolate_rotation,
    })
}

fn read_properties_i8(reader: &mut PacketReader<'_>) -> Result<i8, ProtocolError> {
    Ok(reader.read_u8()? as i8)
}

fn read_properties_i16_pair(reader: &mut PacketReader<'_>) -> Result<[i16; 2], ProtocolError> {
    Ok([reader.read_i16()?, reader.read_i16()?])
}

fn read_properties_argb(reader: &mut PacketReader<'_>) -> Result<[u8; 4], ProtocolError> {
    let bytes = reader.read_bytes(4)?;
    let mut color = [0; 4];
    color.copy_from_slice(bytes);
    Ok(color)
}

fn read_properties_string16(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let bytes = reader.read_string16()?;

    String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidActiveObject("object property string is not UTF-8"))
}

fn read_properties_vec3(reader: &mut PacketReader<'_>) -> Result<Vec3, ProtocolError> {
    Ok(Vec3::new(
        reader.read_f32()?,
        reader.read_f32()?,
        reader.read_f32()?,
    ))
}

pub fn decode_generic_object_properties(
    data: &[u8],
) -> Result<GenericObjectProperties, ProtocolError> {
    if data.len() > MAX_ACTIVE_OBJECT_INIT_DATA_SIZE {
        return Err(ProtocolError::InvalidActiveObject(
            "object properties exceed size limit",
        ));
    }

    let mut reader = PacketReader::new(data);

    let version = reader.read_u8()?;

    if version != 4 {
        return Err(ProtocolError::InvalidActiveObject(
            "unsupported object properties version",
        ));
    }

    let hp_max = reader.read_u16()?;
    let physical = reader.read_u8()? != 0;

    reader.read_u32()?;

    let collisionbox_min = read_properties_vec3(&mut reader)?;
    let collisionbox_max = read_properties_vec3(&mut reader)?;
    let selectionbox_min = read_properties_vec3(&mut reader)?;
    let selectionbox_max = read_properties_vec3(&mut reader)?;

    let pointable = reader.read_u8()?;
    let pointable = if pointable <= 2 { pointable } else { 1 };
    let visual = read_properties_string16(&mut reader)?;
    let visual_size = read_properties_vec3(&mut reader)?;

    let texture_count = usize::from(reader.read_u16()?);

    if texture_count > reader.remaining() / 2 {
        return Err(ProtocolError::InvalidActiveObject(
            "object texture count exceeds packet size",
        ));
    }

    let mut textures = Vec::with_capacity(texture_count);

    for _ in 0..texture_count {
        textures.push(read_properties_string16(&mut reader)?);
    }

    let spritediv = read_properties_i16_pair(&mut reader)?;
    let initial_sprite_basepos = read_properties_i16_pair(&mut reader)?;
    let is_visible = reader.read_u8()? != 0;
    let makes_footstep_sound = reader.read_u8()? != 0;
    let automatic_rotate = reader.read_f32()?;
    let mesh = read_properties_string16(&mut reader)?;

    let color_count = usize::from(reader.read_u16()?);

    if color_count > reader.remaining() / 4 {
        return Err(ProtocolError::InvalidActiveObject(
            "object color count exceeds packet size",
        ));
    }

    let mut colors = Vec::with_capacity(color_count);

    for _ in 0..color_count {
        colors.push(read_properties_argb(&mut reader)?);
    }

    let collide_with_objects = reader.read_u8()? != 0;
    let stepheight = reader.read_f32()?;
    let automatic_face_movement_dir = reader.read_u8()?;
    let automatic_face_movement_dir_offset = reader.read_f32()?;
    let backface_culling = reader.read_u8()? != 0;
    let nametag = read_properties_string16(&mut reader)?;
    let nametag_color = read_properties_argb(&mut reader)?;
    let automatic_face_movement_max_rotation_per_sec = reader.read_f32()?;
    let infotext = read_properties_string16(&mut reader)?;
    let wield_item = read_properties_string16(&mut reader)?;
    let glow = read_properties_i8(&mut reader)?;
    let breath_max = reader.read_u16()?;
    let eye_height = reader.read_f32()?;
    let zoom_fov = reader.read_f32()?;
    let use_texture_alpha = reader.read_u8()?;

    let (
        damage_texture_modifier,
        shaded,
        show_on_minimap,
        nametag_bgcolor,
        rotate_selectionbox,
        node_content,
        node_param1,
        node_param2,
        nametag_fontsize,
        nametag_scale_z,
        step_up_mode,
    ) = if reader.is_empty() {
        (
            None, None, None, None, None, None, None, None, None, None, None,
        )
    } else {
        let damage_texture_modifier = Some(read_properties_string16(&mut reader)?);
        let shaded = Some(reader.read_u8()? != 0);

        if reader.is_empty() {
            (
                damage_texture_modifier,
                shaded,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
        } else {
            let show_on_minimap = Some(reader.read_u8()? != 0);
            let bgcolor = read_properties_argb(&mut reader)?;

            let nametag_bgcolor = Some(if bgcolor == [0, 1, 1, 1] {
                None
            } else {
                Some(bgcolor)
            });

            if reader.is_empty() {
                (
                    damage_texture_modifier,
                    shaded,
                    show_on_minimap,
                    nametag_bgcolor,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
            } else {
                let rotate_selectionbox = Some(reader.read_u8()? != 0);

                if reader.is_empty() {
                    (
                        damage_texture_modifier,
                        shaded,
                        show_on_minimap,
                        nametag_bgcolor,
                        rotate_selectionbox,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                } else {
                    let node_content = Some(reader.read_u16()?);
                    let node_param1 = Some(reader.read_u8()?);
                    let node_param2 = Some(reader.read_u8()?);

                    if reader.is_empty() {
                        (
                            damage_texture_modifier,
                            shaded,
                            show_on_minimap,
                            nametag_bgcolor,
                            rotate_selectionbox,
                            node_content,
                            node_param1,
                            node_param2,
                            None,
                            None,
                            None,
                        )
                    } else {
                        let fontsize = reader.read_u32()?;
                        let nametag_fontsize = Some(if fontsize == u32::MAX {
                            None
                        } else {
                            Some(fontsize)
                        });
                        let nametag_scale_z = Some(reader.read_u8()?);

                        let step_up_mode = if reader.is_empty() {
                            None
                        } else {
                            Some(reader.read_u8()?)
                        };

                        (
                            damage_texture_modifier,
                            shaded,
                            show_on_minimap,
                            nametag_bgcolor,
                            rotate_selectionbox,
                            node_content,
                            node_param1,
                            node_param2,
                            nametag_fontsize,
                            nametag_scale_z,
                            step_up_mode,
                        )
                    }
                }
            }
        }
    };

    let extensions = reader.read_bytes(reader.remaining())?.to_vec();

    Ok(GenericObjectProperties {
        version,
        hp_max,
        physical,
        collisionbox_min,
        collisionbox_max,
        selectionbox_min,
        selectionbox_max,
        pointable,
        visual,
        visual_size,
        textures,
        spritediv,
        initial_sprite_basepos,
        is_visible,
        makes_footstep_sound,
        automatic_rotate,
        mesh,
        colors,
        collide_with_objects,
        stepheight,
        automatic_face_movement_dir,
        automatic_face_movement_dir_offset,
        backface_culling,
        nametag,
        nametag_color,
        automatic_face_movement_max_rotation_per_sec,
        infotext,
        wield_item,
        glow,
        breath_max,
        eye_height,
        zoom_fov,
        use_texture_alpha,
        damage_texture_modifier,
        shaded,
        show_on_minimap,
        nametag_bgcolor,
        rotate_selectionbox,
        node_content,
        node_param1,
        node_param2,
        nametag_fontsize,
        nametag_scale_z,
        step_up_mode,
        extensions,
    })
}
fn read_init_string16(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let bytes = reader.read_string16()?;

    String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidActiveObject("invalid object name"))
}

fn read_init_string32(reader: &mut PacketReader<'_>) -> Result<Vec<u8>, ProtocolError> {
    let length =
        usize::try_from(reader.read_u32()?).map_err(|_| ProtocolError::InvalidStringLength)?;

    if length > MAX_ACTIVE_OBJECT_INIT_DATA_SIZE {
        return Err(ProtocolError::InvalidActiveObject(
            "initialization message exceeds size limit",
        ));
    }

    Ok(reader.read_bytes(length)?.to_vec())
}

fn read_init_vec3(reader: &mut PacketReader<'_>) -> Result<Vec3, ProtocolError> {
    Ok(Vec3::new(
        reader.read_f32()?,
        reader.read_f32()?,
        reader.read_f32()?,
    ))
}

pub fn decode_generic_active_object_message(
    data: &[u8],
) -> Result<GenericActiveObjectMessage, ProtocolError> {
    let (command, payload) = data
        .split_first()
        .ok_or(ProtocolError::InvalidActiveObject(
            "generic object message is empty",
        ))?;

    Ok(GenericActiveObjectMessage {
        command: *command,
        payload: payload.to_vec(),
    })
}

pub fn decode_generic_active_object_init(
    data: &[u8],
) -> Result<GenericActiveObjectInit, ProtocolError> {
    if data.len() > MAX_ACTIVE_OBJECT_INIT_DATA_SIZE {
        return Err(ProtocolError::InvalidActiveObject(
            "initialization data exceeds size limit",
        ));
    }

    let mut reader = PacketReader::new(data);

    let version = reader.read_u8()?;

    if version == 0 {
        return Err(ProtocolError::InvalidActiveObject(
            "unsupported initialization data version",
        ));
    }

    let name = read_init_string16(&mut reader)?;
    let is_player = reader.read_u8()? != 0;
    let id = reader.read_u16()?;
    let position = read_init_vec3(&mut reader)?;
    let rotation = read_init_vec3(&mut reader)?;
    let hp = reader.read_u16()?;

    let message_count = usize::from(reader.read_u8()?);
    let mut messages = Vec::with_capacity(message_count);

    for _ in 0..message_count {
        let message = read_init_string32(&mut reader)?;
        messages.push(decode_generic_active_object_message(&message)?);
    }

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after object initialization data",
        ));
    }

    Ok(GenericActiveObjectInit {
        version,
        name,
        is_player,
        id,
        position,
        rotation,
        hp,
        messages,
    })
}

pub fn decode_active_object_remove_add(data: &[u8]) -> Result<ActiveObjectUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let removed_count = usize::from(reader.read_u16()?);

    if removed_count > reader.remaining() / 2 {
        return Err(ProtocolError::InvalidActiveObject(
            "removed object count exceeds packet size",
        ));
    }

    let mut removed = Vec::with_capacity(removed_count);

    for _ in 0..removed_count {
        removed.push(reader.read_u16()?);
    }

    let added_count = usize::from(reader.read_u16()?);

    if added_count > reader.remaining() / 7 {
        return Err(ProtocolError::InvalidActiveObject(
            "added object count exceeds packet size",
        ));
    }

    let mut added = Vec::with_capacity(added_count);

    for _ in 0..added_count {
        let id = reader.read_u16()?;
        let object_type = reader.read_u8()?;
        let init_length =
            usize::try_from(reader.read_u32()?).map_err(|_| ProtocolError::InvalidStringLength)?;

        if init_length > MAX_ACTIVE_OBJECT_INIT_DATA_SIZE {
            return Err(ProtocolError::InvalidActiveObject(
                "initialization data exceeds size limit",
            ));
        }

        let initialization_data = reader.read_bytes(init_length)?.to_vec();

        added.push(ActiveObjectData {
            id,
            object_type,
            initialization_data,
        });
    }

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidActiveObject(
            "unexpected bytes after active object update",
        ));
    }

    Ok(ActiveObjectUpdate { removed, added })
}

pub fn decode_active_object_messages(
    data: &[u8],
) -> Result<Vec<ActiveObjectMessage>, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let mut messages = Vec::new();

    while !reader.is_empty() {
        let id = reader.read_u16()?;
        let length = usize::from(reader.read_u16()?);
        let message = reader.read_bytes(length)?.to_vec();

        messages.push(ActiveObjectMessage { id, data: message });
    }

    Ok(messages)
}
