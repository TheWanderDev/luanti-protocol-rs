use crate::errors::ProtocolError;
use crate::protocol::reader::PacketReader;
use crate::protocol::vectors::read_vec3_f32;
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoExposureUpdate {
    pub luminance_min: f32,
    pub luminance_max: f32,
    pub exposure_correction: f32,
    pub speed_dark_bright: f32,
    pub speed_bright_dark: f32,
    pub center_weight_power: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightingUpdate {
    pub shadow_intensity: f32,
    pub saturation: f32,
    pub exposure: AutoExposureUpdate,
    pub volumetric_light_strength: f32,
    pub shadow_tint: [u8; 4],
    pub bloom_intensity: f32,
    pub bloom_strength_factor: f32,
    pub bloom_radius: f32,
    pub shadow_direction: Vec3,
    pub light_curve: Option<[u8; 16]>,
    pub ambient_occlusion_gamma: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaySoundUpdate {
    pub server_id: i32,
    pub name: String,
    pub gain: f32,
    pub location_type: u8,
    pub position: Vec3,
    pub object_id: u16,
    pub looped: bool,
    pub fade: f32,
    pub pitch: f32,
    pub ephemeral: bool,
    pub start_time: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopSoundUpdate {
    pub sound_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeSoundUpdate {
    pub sound_id: i32,
    pub step: f32,
    pub gain: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivilegesUpdate {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerListOperation {
    Init,
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListUpdate {
    pub operation: PlayerListOperation,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelativePlayerMove {
    pub delta: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FovUpdate {
    pub fov: f32,
    pub is_multiplier: bool,
    pub transition_time: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraModeUpdate {
    pub allowed_mode: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsmRestrictionFlagsUpdate {
    pub flags: u64,
    pub node_range: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudFlagsUpdate {
    pub flags: u32,
    pub mask: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudParamUpdate {
    pub param: u16,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeleteParticleSpawner {
    pub id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayNightRatioUpdate {
    pub enabled: bool,
    pub ratio: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EyeOffsetUpdate {
    pub first_person: Vec3,
    pub third_person: Vec3,
    pub third_person_front: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CloudParamsUpdate {
    pub density: f32,
    pub color_diffuse: [u8; 4],
    pub color_ambient: [u8; 4],
    pub height: f32,
    pub thickness: f32,
    pub speed: glam::Vec2,
    pub shadow: Option<[u8; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SunUpdate {
    pub visible: bool,
    pub texture: String,
    pub tonemap: String,
    pub sunrise: String,
    pub sunrise_visible: bool,
    pub scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoonUpdate {
    pub visible: bool,
    pub texture: String,
    pub tonemap: String,
    pub scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StarsUpdate {
    pub visible: bool,
    pub count: u32,
    pub color: [u8; 4],
    pub scale: f32,
    pub day_opacity: f32,
    pub star_seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkyColors {
    pub day_sky: [u8; 4],
    pub day_horizon: [u8; 4],
    pub dawn_sky: [u8; 4],
    pub dawn_horizon: [u8; 4],
    pub night_sky: [u8; 4],
    pub night_horizon: [u8; 4],
    pub indoors: [u8; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkyUpdate {
    pub base_color: [u8; 4],
    pub sky_type: String,
    pub texture_count: u16,
    pub textures: Vec<String>,
    pub clouds: bool,
    pub fog_sun_tint: Option<[u8; 4]>,
    pub fog_moon_tint: Option<[u8; 4]>,
    pub fog_tint_type: Option<String>,
    pub colors: Option<SkyColors>,
    pub body_orbit_tilt: Option<f32>,
    pub fog_distance: Option<i16>,
    pub fog_start: Option<f32>,
    pub fog_color: Option<[u8; 4]>,
    pub auto_dim_skybox: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalPlayerAnimationsUpdate {
    pub stand: glam::Vec2,
    pub walk: glam::Vec2,
    pub dig: glam::Vec2,
    pub walk_dig: glam::Vec2,
    pub frame_speed: f32,
}

fn decode_legacy_sky(reader: &mut PacketReader<'_>) -> Result<SkyUpdate, ProtocolError> {
    let base_color = read_environment_color(reader)?;
    let sky_type = read_environment_string16(reader)?;
    let texture_count = reader.read_u16()?;
    let texture_capacity = usize::from(texture_count);

    if texture_capacity > reader.remaining() / 2 {
        return Err(ProtocolError::InvalidPlayerValue(
            "sky texture count exceeds packet size",
        ));
    }

    let mut textures = Vec::with_capacity(texture_capacity);

    for _ in 0..texture_count {
        textures.push(read_environment_string16(reader)?);
    }

    let clouds = read_environment_bool(reader)?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue(
            "unexpected bytes after legacy sky parameters",
        ));
    }

    Ok(SkyUpdate {
        base_color,
        sky_type,
        texture_count,
        textures,
        clouds,
        fog_sun_tint: None,
        fog_moon_tint: None,
        fog_tint_type: None,
        colors: None,
        body_orbit_tilt: None,
        fog_distance: None,
        fog_start: None,
        fog_color: None,
        auto_dim_skybox: None,
    })
}

pub(crate) fn decode_lighting(data: &[u8]) -> Result<LightingUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let shadow_intensity = reader.read_f32()?;
    let mut saturation = 1.0;
    let mut exposure = AutoExposureUpdate {
        luminance_min: -3.0,
        luminance_max: -3.0,
        exposure_correction: 0.0,
        speed_dark_bright: 1000.0,
        speed_bright_dark: 1000.0,
        center_weight_power: 1.0,
    };
    let mut volumetric_light_strength = 0.0;
    let mut shadow_tint = [255, 0, 0, 0];
    let mut bloom_intensity = 0.05;
    let mut bloom_strength_factor = 1.0;
    let mut bloom_radius = 1.0;
    let mut shadow_direction = Vec3::ZERO;
    let mut light_curve = None;
    let mut ambient_occlusion_gamma = 1.8;

    if !reader.is_empty() {
        saturation = reader.read_f32()?;
        exposure = AutoExposureUpdate {
            luminance_min: reader.read_f32()?,
            luminance_max: reader.read_f32()?,
            exposure_correction: reader.read_f32()?,
            speed_dark_bright: reader.read_f32()?,
            speed_bright_dark: reader.read_f32()?,
            center_weight_power: reader.read_f32()?,
        };
    }
    if !reader.is_empty() {
        volumetric_light_strength = reader.read_f32()?;
    }
    if !reader.is_empty() {
        shadow_tint = read_environment_color(&mut reader)?;
        bloom_intensity = reader.read_f32()?;
        bloom_strength_factor = reader.read_f32()?;
        bloom_radius = reader.read_f32()?;
    }
    if !reader.is_empty() {
        shadow_direction = read_vec3_f32(&mut reader)?;
    }
    if !reader.is_empty() {
        let has_light_curve = read_environment_bool(&mut reader)?;
        if has_light_curve {
            let mut curve = [0_u8; 16];
            let curve_len = curve.len();
            let curve_bytes = reader.read_bytes(curve_len)?;
            curve.copy_from_slice(curve_bytes);
            light_curve = Some(curve);
        }
        ambient_occlusion_gamma = reader.read_f32()?;
    }

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue(
            "unexpected bytes after lighting parameters",
        ));
    }

    let float_values = [
        shadow_intensity,
        saturation,
        exposure.luminance_min,
        exposure.luminance_max,
        exposure.exposure_correction,
        exposure.speed_dark_bright,
        exposure.speed_bright_dark,
        exposure.center_weight_power,
        volumetric_light_strength,
        bloom_intensity,
        bloom_strength_factor,
        bloom_radius,
        ambient_occlusion_gamma,
    ];

    if !float_values.iter().all(|value| value.is_finite()) || !shadow_direction.is_finite() {
        return Err(ProtocolError::InvalidPlayerValue(
            "invalid lighting parameters",
        ));
    }

    Ok(LightingUpdate {
        shadow_intensity,
        saturation,
        exposure,
        volumetric_light_strength,
        shadow_tint,
        bloom_intensity,
        bloom_strength_factor,
        bloom_radius,
        shadow_direction,
        light_curve,
        ambient_occlusion_gamma,
    })
}

fn decode_modern_sky(reader: &mut PacketReader<'_>) -> Result<SkyUpdate, ProtocolError> {
    let base_color = read_environment_color(reader)?;
    let sky_type = read_environment_string16(reader)?;
    let clouds = read_environment_bool(reader)?;
    let fog_sun_tint = read_environment_color(reader)?;
    let fog_moon_tint = read_environment_color(reader)?;
    let fog_tint_type = read_environment_string16(reader)?;

    let (texture_count, textures, colors) = match sky_type.as_str() {
        "skybox" => {
            let texture_count = reader.read_u16()?;
            let count = usize::from(texture_count);

            if count > reader.remaining() / 2 {
                return Err(ProtocolError::InvalidPlayerValue(
                    "sky texture count exceeds packet size",
                ));
            }

            let mut textures = Vec::with_capacity(count);
            for _ in 0..texture_count {
                textures.push(read_environment_string16(reader)?);
            }

            (texture_count, textures, None)
        }
        "regular" => {
            let colors = SkyColors {
                day_sky: read_environment_color(reader)?,
                day_horizon: read_environment_color(reader)?,
                dawn_sky: read_environment_color(reader)?,
                dawn_horizon: read_environment_color(reader)?,
                night_sky: read_environment_color(reader)?,
                night_horizon: read_environment_color(reader)?,
                indoors: read_environment_color(reader)?,
            };

            (0, Vec::new(), Some(colors))
        }
        _ => (0, Vec::new(), None),
    };

    let mut body_orbit_tilt = None;
    let mut fog_distance = None;
    let mut fog_start = None;
    let mut fog_color = None;
    let mut auto_dim_skybox = None;

    if !reader.is_empty() {
        body_orbit_tilt = Some(reader.read_f32()?);
    }
    if !reader.is_empty() {
        fog_distance = Some(reader.read_i16()?);
        fog_start = Some(reader.read_f32()?);
    }
    if !reader.is_empty() {
        fog_color = Some(read_environment_color(reader)?);
    }
    if !reader.is_empty() {
        auto_dim_skybox = Some(read_environment_bool(reader)?);
    }

    if !reader.is_empty()
        || body_orbit_tilt.is_some_and(|value| !value.is_finite())
        || fog_start.is_some_and(|value| !value.is_finite())
    {
        return Err(ProtocolError::InvalidPlayerValue("invalid sky parameters"));
    }

    Ok(SkyUpdate {
        base_color,
        sky_type,
        texture_count,
        textures,
        clouds,
        fog_sun_tint: Some(fog_sun_tint),
        fog_moon_tint: Some(fog_moon_tint),
        fog_tint_type: Some(fog_tint_type),
        colors,
        body_orbit_tilt,
        fog_distance,
        fog_start,
        fog_color,
        auto_dim_skybox,
    })
}

pub(crate) fn decode_sky(data: &[u8], protocol_version: u16) -> Result<SkyUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    if protocol_version < 39 {
        decode_legacy_sky(&mut reader)
    } else {
        decode_modern_sky(&mut reader)
    }
}

pub(crate) fn decode_local_player_animations(
    data: &[u8],
    protocol_version: u16,
) -> Result<LocalPlayerAnimationsUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let read_frame_pair = |reader: &mut PacketReader<'_>| -> Result<glam::Vec2, ProtocolError> {
        if protocol_version >= 46 {
            Ok(glam::Vec2::new(reader.read_f32()?, reader.read_f32()?))
        } else {
            Ok(glam::IVec2::new(reader.read_i32()?, reader.read_i32()?).as_vec2())
        }
    };

    let stand = read_frame_pair(&mut reader)?;
    let walk = read_frame_pair(&mut reader)?;
    let dig = read_frame_pair(&mut reader)?;
    let walk_dig = read_frame_pair(&mut reader)?;

    let frame_speed = reader.read_f32()?;
    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue(
            "unexpected bytes after local player animations",
        ));
    }

    if !stand.is_finite()
        || !walk.is_finite()
        || !dig.is_finite()
        || !walk_dig.is_finite()
        || !frame_speed.is_finite()
    {
        return Err(ProtocolError::InvalidPlayerValue(
            "invalid local player animation values",
        ));
    }

    Ok(LocalPlayerAnimationsUpdate {
        stand,
        walk,
        dig,
        walk_dig,
        frame_speed,
    })
}

fn read_environment_bool(reader: &mut PacketReader<'_>) -> Result<bool, ProtocolError> {
    match reader.read_u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(ProtocolError::InvalidPlayerValue(
            "invalid environment boolean",
        )),
    }
}

fn read_environment_color(reader: &mut PacketReader<'_>) -> Result<[u8; 4], ProtocolError> {
    let bytes = reader.read_bytes(4)?;
    Ok([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn read_environment_string16(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let bytes = reader.read_string16()?;

    String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidPlayerValue("environment string is not UTF-8"))
}

pub(crate) fn decode_day_night_ratio(data: &[u8]) -> Result<DayNightRatioUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let enabled = read_environment_bool(&mut reader)?;
    let ratio = reader.read_u16()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue(
            "unexpected bytes after day night ratio",
        ));
    }

    Ok(DayNightRatioUpdate { enabled, ratio })
}

pub(crate) fn decode_eye_offset(data: &[u8]) -> Result<EyeOffsetUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let first_person = read_vec3_f32(&mut reader)?;
    let third_person = read_vec3_f32(&mut reader)?;
    let third_person_front = if reader.is_empty() {
        third_person
    } else {
        read_vec3_f32(&mut reader)?
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue(
            "unexpected bytes after eye offset",
        ));
    }

    if !first_person.is_finite() || !third_person.is_finite() || !third_person_front.is_finite() {
        return Err(ProtocolError::InvalidPlayerValue(
            "invalid eye offset values",
        ));
    }

    Ok(EyeOffsetUpdate {
        first_person,
        third_person,
        third_person_front,
    })
}

pub(crate) fn decode_cloud_params(data: &[u8]) -> Result<CloudParamsUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let density = reader.read_f32()?;
    let color_diffuse = read_environment_color(&mut reader)?;
    let color_ambient = read_environment_color(&mut reader)?;
    let height = reader.read_f32()?;
    let thickness = reader.read_f32()?;
    let speed = glam::Vec2::new(reader.read_f32()?, reader.read_f32()?);

    let shadow = match reader.remaining() {
        0 => None,
        4 => Some(read_environment_color(&mut reader)?),
        _ => {
            return Err(ProtocolError::InvalidPlayerValue(
                "invalid cloud shadow extension length",
            ));
        }
    };

    if !density.is_finite() || !height.is_finite() || !thickness.is_finite() || !speed.is_finite() {
        return Err(ProtocolError::InvalidPlayerValue(
            "invalid cloud parameters",
        ));
    }

    Ok(CloudParamsUpdate {
        density,
        color_diffuse,
        color_ambient,
        height,
        thickness,
        speed,
        shadow,
    })
}

pub(crate) fn decode_sun(data: &[u8]) -> Result<SunUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let visible = read_environment_bool(&mut reader)?;
    let texture = read_environment_string16(&mut reader)?;
    let tonemap = read_environment_string16(&mut reader)?;
    let sunrise = read_environment_string16(&mut reader)?;
    let sunrise_visible = read_environment_bool(&mut reader)?;
    let scale = reader.read_f32()?;

    if !scale.is_finite() || !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue("invalid sun parameters"));
    }

    Ok(SunUpdate {
        visible,
        texture,
        tonemap,
        sunrise,
        sunrise_visible,
        scale,
    })
}

pub(crate) fn decode_moon(data: &[u8]) -> Result<MoonUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let visible = read_environment_bool(&mut reader)?;
    let texture = read_environment_string16(&mut reader)?;
    let tonemap = read_environment_string16(&mut reader)?;
    let scale = reader.read_f32()?;

    if !scale.is_finite() || !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue("invalid moon parameters"));
    }

    Ok(MoonUpdate {
        visible,
        texture,
        tonemap,
        scale,
    })
}

pub(crate) fn decode_stars(data: &[u8]) -> Result<StarsUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let visible = read_environment_bool(&mut reader)?;
    let count = reader.read_u32()?;
    let color = read_environment_color(&mut reader)?;
    let scale = reader.read_f32()?;
    let day_opacity = if reader.is_empty() {
        0.0
    } else {
        reader.read_f32()?
    };
    let star_seed = if reader.is_empty() {
        None
    } else {
        Some(reader.read_u64()?)
    };

    if !scale.is_finite() || !day_opacity.is_finite() || !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue(
            "invalid stars parameters",
        ));
    }

    Ok(StarsUpdate {
        visible,
        count,
        color,
        scale,
        day_opacity,
        star_seed,
    })
}

fn read_server_name(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    let bytes = reader.read_string16()?;

    String::from_utf8(bytes.to_vec())
        .map_err(|_| ProtocolError::InvalidPlayerValue("name is not UTF-8"))
}

fn read_server_name_list(reader: &mut PacketReader<'_>) -> Result<Vec<String>, ProtocolError> {
    let count = usize::from(reader.read_u16()?);

    if count > reader.remaining() / 2 {
        return Err(ProtocolError::InvalidPlayerValue(
            "invalid name entry count",
        ));
    }

    let mut names = Vec::with_capacity(count);

    for _ in 0..count {
        names.push(read_server_name(reader)?);
    }

    Ok(names)
}

pub(crate) fn decode_play_sound(data: &[u8]) -> Result<PlaySoundUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let server_id = reader.read_i32()?;
    let name = String::from_utf8(reader.read_string16()?.to_vec())
        .map_err(|_| ProtocolError::InvalidSound("sound name is not UTF-8"))?;
    let gain = reader.read_f32()?;
    let location_type = reader.read_u8()?;
    let position = read_vec3_f32(&mut reader)?;
    let object_id = reader.read_u16()?;
    let looped = reader.read_u8()? != 0;
    let fade = reader.read_f32()?;
    let pitch = reader.read_f32()?;

    let ephemeral = if reader.is_empty() {
        false
    } else {
        reader.read_u8()? != 0
    };
    let start_time = if reader.is_empty() {
        0.0
    } else {
        reader.read_f32()?
    };

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidSound(
            "unexpected bytes after play sound",
        ));
    }

    if !gain.is_finite()
        || !position.is_finite()
        || !fade.is_finite()
        || !pitch.is_finite()
        || !start_time.is_finite()
    {
        return Err(ProtocolError::InvalidSound("invalid play sound values"));
    }

    Ok(PlaySoundUpdate {
        server_id,
        name,
        gain,
        location_type,
        position,
        object_id,
        looped,
        fade,
        pitch,
        ephemeral,
        start_time,
    })
}

pub(crate) fn decode_stop_sound(data: &[u8]) -> Result<StopSoundUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let sound_id = reader.read_i32()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue(
            "unexpected bytes after stop sound",
        ));
    }

    Ok(StopSoundUpdate { sound_id })
}

pub(crate) fn decode_fade_sound(data: &[u8]) -> Result<FadeSoundUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let sound_id = reader.read_i32()?;
    let step = reader.read_f32()?;
    let gain = reader.read_f32()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue(
            "unexpected bytes after fade sound",
        ));
    }

    if !step.is_finite() || !gain.is_finite() {
        return Err(ProtocolError::InvalidPlayerValue(
            "invalid fade sound values",
        ));
    }

    Ok(FadeSoundUpdate {
        sound_id,
        step,
        gain,
    })
}

pub(crate) fn decode_privileges(data: &[u8]) -> Result<PrivilegesUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let names = read_server_name_list(&mut reader)?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue(
            "unexpected bytes after privileges",
        ));
    }

    Ok(PrivilegesUpdate { names })
}

pub(crate) fn decode_player_list(data: &[u8]) -> Result<PlayerListUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let operation = match reader.read_u8()? {
        0 => PlayerListOperation::Init,
        1 => PlayerListOperation::Add,
        2 => PlayerListOperation::Remove,
        _ => {
            return Err(ProtocolError::InvalidPlayerValue(
                "unknown player list operation",
            ));
        }
    };

    let names = read_server_name_list(&mut reader)?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue(
            "unexpected bytes after player list",
        ));
    }

    Ok(PlayerListUpdate { operation, names })
}

pub(crate) fn decode_relative_player_move(
    data: &[u8],
) -> Result<RelativePlayerMove, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let delta = read_vec3_f32(&mut reader)?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerValue(
            "unexpected bytes after relative movement",
        ));
    }

    if !delta.is_finite() {
        return Err(ProtocolError::InvalidPlayerValue(
            "invalid relative movement vector",
        ));
    }

    Ok(RelativePlayerMove { delta })
}

pub(crate) fn decode_fov(data: &[u8]) -> Result<FovUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let fov = reader.read_f32()?;
    let multiplier = reader.read_u8()?;
    let transition_time = if reader.is_empty() {
        0.0
    } else {
        reader.read_f32()?
    };

    if multiplier > 1 || !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerPacket);
    }

    if !fov.is_finite() || !transition_time.is_finite() || transition_time < 0.0 {
        return Err(ProtocolError::InvalidPlayerPacket);
    }

    Ok(FovUpdate {
        fov,
        is_multiplier: multiplier != 0,
        transition_time,
    })
}

pub(crate) fn decode_csm_restriction_flags(
    data: &[u8],
) -> Result<CsmRestrictionFlagsUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let flags = reader.read_u64()?;
    let node_range = reader.read_u32()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerPacket);
    }

    Ok(CsmRestrictionFlagsUpdate { flags, node_range })
}

pub(crate) fn decode_camera_mode(data: &[u8]) -> Result<CameraModeUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let allowed_mode = reader.read_u8()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerPacket);
    }

    Ok(CameraModeUpdate { allowed_mode })
}
pub(crate) fn decode_hud_flags(data: &[u8]) -> Result<HudFlagsUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let flags = reader.read_u32()?;
    let mask = reader.read_u32()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidHud("trailing packet data"));
    }

    Ok(HudFlagsUpdate { flags, mask })
}

pub(crate) fn decode_hud_param(data: &[u8]) -> Result<HudParamUpdate, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let param = reader.read_u16()?;
    let value = reader.read_string16()?.to_vec();

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidHud("trailing packet data"));
    }

    Ok(HudParamUpdate { param, value })
}

pub(crate) fn decode_delete_particle_spawner(
    data: &[u8],
) -> Result<DeleteParticleSpawner, ProtocolError> {
    let mut reader = PacketReader::new(data);

    let id = reader.read_u32()?;

    if !reader.is_empty() {
        return Err(ProtocolError::InvalidPlayerPacket);
    }

    Ok(DeleteParticleSpawner { id })
}
