use glam::{Vec2, Vec3};

use crate::errors::ProtocolError;
use crate::protocol::compression::decompress_zstd_limited;
use crate::protocol::reader::PacketReader;
use crate::protocol::vectors::read_vec3_f32;

const MAX_PARTICLE_BATCH_SIZE: usize = 16_700_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleBlendMode {
    Alpha,
    Add,
    Subtract,
    Screen,
    Clip,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleAnimation {
    None,
    VerticalFrames {
        aspect_width: u16,
        aspect_height: u16,
        length: f32,
    },
    Sheet2d {
        frames_width: u8,
        frames_height: u8,
        frame_length: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleRangeVec3 {
    pub min: Vec3,
    pub max: Vec3,
    pub bias: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleRangeF32 {
    pub min: f32,
    pub max: f32,
    pub bias: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleTween<T> {
    pub style: u8,
    pub repetitions: u16,
    pub beginning: f32,
    pub start: T,
    pub end: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleTexture {
    pub name: String,
    pub animated: bool,
    pub blend_mode: ParticleBlendMode,
    pub alpha: ParticleTween<f32>,
    pub scale: ParticleTween<Vec2>,
    pub animation: Option<ParticleAnimation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleCommonParameters {
    pub collision_detection: bool,
    pub collision_removal: bool,
    pub object_collision: bool,
    pub vertical: bool,
    pub texture: ParticleTexture,
    pub animation: ParticleAnimation,
    pub glow: u8,
    pub node_param0: u16,
    pub node_param2: u8,
    pub node_tile: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleParameters {
    pub common: ParticleCommonParameters,
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub drag: Vec3,
    pub expiration_time: f32,
    pub size: f32,
    pub jitter: ParticleRangeVec3,
    pub bounce: ParticleRangeF32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleSpawnerParameters {
    pub id: u32,
    pub attached_id: u16,
    pub amount: u16,
    pub time: f32,
    pub common: ParticleCommonParameters,
    pub position: ParticleTween<ParticleRangeVec3>,
    pub velocity: ParticleTween<ParticleRangeVec3>,
    pub acceleration: ParticleTween<ParticleRangeVec3>,
    pub expiration_time: ParticleTween<ParticleRangeF32>,
    pub size: ParticleTween<ParticleRangeF32>,
    pub drag: ParticleTween<ParticleRangeVec3>,
    pub jitter: ParticleTween<ParticleRangeVec3>,
    pub bounce: ParticleTween<ParticleRangeF32>,
    pub attractor_kind: u8,
    pub attract: Option<ParticleTween<ParticleRangeF32>>,
    pub attractor_origin: Option<ParticleTween<Vec3>>,
    pub attractor_attachment: u16,
    pub attractor_kill: bool,
    pub attractor_direction: Option<ParticleTween<Vec3>>,
    pub attractor_direction_attachment: u16,
    pub radius: Option<ParticleTween<ParticleRangeVec3>>,
    pub texture_pool: Vec<ParticleTexture>,
}

fn invalid_particle(reason: &'static str) -> ProtocolError {
    ProtocolError::InvalidParticle(reason)
}

fn read_finite_f32(reader: &mut PacketReader<'_>) -> Result<f32, ProtocolError> {
    let value = reader.read_f32()?;

    if !value.is_finite() {
        return Err(invalid_particle("overflow floating-point value"));
    }

    Ok(value)
}

fn read_finite_vec2(reader: &mut PacketReader<'_>) -> Result<Vec2, ProtocolError> {
    let value = Vec2::new(read_finite_f32(reader)?, read_finite_f32(reader)?);
    Ok(value)
}

fn read_finite_vec3(reader: &mut PacketReader<'_>) -> Result<Vec3, ProtocolError> {
    let value = read_vec3_f32(reader)?;

    if !value.is_finite() {
        return Err(invalid_particle("overflow vector value"));
    }

    Ok(value)
}

fn read_particle_bool(reader: &mut PacketReader<'_>) -> Result<bool, ProtocolError> {
    Ok(reader.read_u8()? != 0)
}

fn read_particle_string(reader: &mut PacketReader<'_>) -> Result<String, ProtocolError> {
    String::from_utf8(reader.read_string32()?.to_vec())
        .map_err(|_| invalid_particle("invalid texture name encoding"))
}

fn read_animation(reader: &mut PacketReader<'_>) -> Result<ParticleAnimation, ProtocolError> {
    match reader.read_u8()? {
        0 => Ok(ParticleAnimation::None),
        1 => {
            let aspect_width = reader.read_u16()?;
            let aspect_height = reader.read_u16()?;
            let length = read_finite_f32(reader)?;

            if aspect_width == 0 || aspect_height == 0 {
                return Err(invalid_particle("invalid vertical particle animation"));
            }

            Ok(ParticleAnimation::VerticalFrames {
                aspect_width,
                aspect_height,
                length,
            })
        }
        2 => {
            let frames_width = reader.read_u8()?;
            let frames_height = reader.read_u8()?;
            let frame_length = read_finite_f32(reader)?;

            if frames_width == 0 || frames_height == 0 {
                return Err(invalid_particle("invalid sheet particle animation"));
            }

            Ok(ParticleAnimation::Sheet2d {
                frames_width,
                frames_height,
                frame_length,
            })
        }
        _ => Ok(ParticleAnimation::None),
    }
}

fn read_range_vec3(
    reader: &mut PacketReader<'_>,
    with_bias: bool,
) -> Result<ParticleRangeVec3, ProtocolError> {
    let min = read_finite_vec3(reader)?;
    let max = read_finite_vec3(reader)?;
    let bias = if with_bias {
        read_finite_f32(reader)?
    } else {
        0.0
    };

    Ok(ParticleRangeVec3 { min, max, bias })
}

fn read_range_f32(
    reader: &mut PacketReader<'_>,
    with_bias: bool,
) -> Result<ParticleRangeF32, ProtocolError> {
    let min = read_finite_f32(reader)?;
    let max = read_finite_f32(reader)?;
    let bias = if with_bias {
        read_finite_f32(reader)?
    } else {
        0.0
    };

    Ok(ParticleRangeF32 { min, max, bias })
}

fn read_tween_header(reader: &mut PacketReader<'_>) -> Result<(u8, u16, f32), ProtocolError> {
    let style = reader.read_u8()?;
    let style = if style <= 3 { style } else { 0 };
    let repetitions = reader.read_u16()?;
    let beginning = read_finite_f32(reader)?;

    Ok((style, repetitions, beginning))
}

fn read_tween_vec2(reader: &mut PacketReader<'_>) -> Result<ParticleTween<Vec2>, ProtocolError> {
    let (style, repetitions, beginning) = read_tween_header(reader)?;
    let start = read_finite_vec2(reader)?;
    let end = read_finite_vec2(reader)?;

    Ok(ParticleTween {
        style,
        repetitions,
        beginning,
        start,
        end,
    })
}

fn read_tween_vec3(reader: &mut PacketReader<'_>) -> Result<ParticleTween<Vec3>, ProtocolError> {
    let (style, repetitions, beginning) = read_tween_header(reader)?;
    let start = read_finite_vec3(reader)?;
    let end = read_finite_vec3(reader)?;

    Ok(ParticleTween {
        style,
        repetitions,
        beginning,
        start,
        end,
    })
}

fn read_tween_f32(reader: &mut PacketReader<'_>) -> Result<ParticleTween<f32>, ProtocolError> {
    let (style, repetitions, beginning) = read_tween_header(reader)?;
    let start = read_finite_f32(reader)?;
    let end = read_finite_f32(reader)?;

    Ok(ParticleTween {
        style,
        repetitions,
        beginning,
        start,
        end,
    })
}

fn read_tween_range_vec3(
    reader: &mut PacketReader<'_>,
) -> Result<ParticleTween<ParticleRangeVec3>, ProtocolError> {
    let (style, repetitions, beginning) = read_tween_header(reader)?;
    let start = read_range_vec3(reader, true)?;
    let end = read_range_vec3(reader, true)?;

    Ok(ParticleTween {
        style,
        repetitions,
        beginning,
        start,
        end,
    })
}

fn read_tween_range_f32(
    reader: &mut PacketReader<'_>,
) -> Result<ParticleTween<ParticleRangeF32>, ProtocolError> {
    let (style, repetitions, beginning) = read_tween_header(reader)?;
    let start = read_range_f32(reader, true)?;
    let end = read_range_f32(reader, true)?;

    Ok(ParticleTween {
        style,
        repetitions,
        beginning,
        start,
        end,
    })
}

fn read_texture_properties(
    reader: &mut PacketReader<'_>,
    name: String,
    include_animation: bool,
) -> Result<ParticleTexture, ProtocolError> {
    let flags = reader.read_u8()?;
    let blend_value = (flags >> 1) & 0x07;
    let blend_mode = decode_blend_mode(blend_value)?;
    let alpha = read_tween_f32(reader)?;
    let scale = read_tween_vec2(reader)?;
    let animated = flags & 1 != 0;
    let animation = if animated && include_animation {
        Some(read_animation(reader)?)
    } else {
        None
    };

    Ok(ParticleTexture {
        name,
        animated,
        blend_mode,
        alpha,
        scale,
        animation,
    })
}

fn decode_blend_mode(value: u8) -> Result<ParticleBlendMode, ProtocolError> {
    match value {
        0 => Ok(ParticleBlendMode::Alpha),
        1 => Ok(ParticleBlendMode::Add),
        2 => Ok(ParticleBlendMode::Subtract),
        3 => Ok(ParticleBlendMode::Screen),
        4 => Ok(ParticleBlendMode::Clip),
        _ => Err(invalid_particle("unsupported particle blend mode")),
    }
}

fn read_texture_pool_entry(
    reader: &mut PacketReader<'_>,
) -> Result<ParticleTexture, ProtocolError> {
    let flags = reader.read_u8()?;
    let blend_mode = decode_blend_mode((flags >> 1) & 0x07)?;
    let alpha = read_tween_f32(reader)?;
    let scale = read_tween_vec2(reader)?;
    let name = read_particle_string(reader)?;
    let animated = flags & 1 != 0;
    let animation = if animated {
        Some(read_animation(reader)?)
    } else {
        None
    };

    Ok(ParticleTexture {
        name,
        animated,
        blend_mode,
        alpha,
        scale,
        animation,
    })
}

pub fn decode_spawn_particle(data: &[u8]) -> Result<ParticleParameters, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let position = read_finite_vec3(&mut reader)?;
    let velocity = read_finite_vec3(&mut reader)?;
    let acceleration = read_finite_vec3(&mut reader)?;
    let expiration_time = read_finite_f32(&mut reader)?;
    let size = read_finite_f32(&mut reader)?;
    let collision_detection = read_particle_bool(&mut reader)?;
    let texture_name = read_particle_string(&mut reader)?;
    let vertical = read_particle_bool(&mut reader)?;
    let collision_removal = read_particle_bool(&mut reader)?;
    let animation = read_animation(&mut reader)?;
    let glow = reader.read_u8()?;
    let object_collision = read_particle_bool(&mut reader)?;

    let mut node_param0 = 0;
    let mut node_param2 = 0;
    let mut node_tile = 0;
    let mut drag = Vec3::ZERO;
    let mut jitter = ParticleRangeVec3 {
        min: Vec3::ZERO,
        max: Vec3::ZERO,
        bias: 0.0,
    };
    let mut bounce = ParticleRangeF32 {
        min: 0.0,
        max: 0.0,
        bias: 0.0,
    };
    let mut texture = ParticleTexture {
        name: texture_name,
        animated: false,
        blend_mode: ParticleBlendMode::Alpha,
        alpha: ParticleTween {
            style: 0,
            repetitions: 1,
            beginning: 0.0,
            start: 1.0,
            end: 1.0,
        },
        scale: ParticleTween {
            style: 0,
            repetitions: 1,
            beginning: 0.0,
            start: Vec2::ONE,
            end: Vec2::ONE,
        },
        animation: None,
    };

    if !reader.is_empty() {
        node_param0 = reader.read_u16()?;
        node_param2 = reader.read_u8()?;
        node_tile = reader.read_u8()?;
    }

    if !reader.is_empty() {
        drag = read_finite_vec3(&mut reader)?;
        jitter = read_range_vec3(&mut reader, true)?;
        bounce = read_range_f32(&mut reader, true)?;
    }

    if !reader.is_empty() {
        let properties = read_texture_properties(&mut reader, texture.name.clone(), false)?;
        texture = properties;
    }

    if !reader.is_empty() {
        return Err(invalid_particle(
            "unexpected bytes after particle parameters",
        ));
    }

    Ok(ParticleParameters {
        common: ParticleCommonParameters {
            collision_detection,
            collision_removal,
            object_collision,
            vertical,
            texture,
            animation,
            glow,
            node_param0,
            node_param2,
            node_tile,
        },
        position,
        velocity,
        acceleration,
        drag,
        expiration_time,
        size,
        jitter,
        bounce,
    })
}

pub fn decode_add_particle_spawner(
    data: &[u8],
    protocol_version: u16,
) -> Result<ParticleSpawnerParameters, ProtocolError> {
    let mut reader = PacketReader::new(data);
    let amount = reader.read_u16()?;
    let time = read_finite_f32(&mut reader)?;

    if time < 0.0 {
        return Err(invalid_particle("negative particle spawner duration"));
    }

    let (mut position, mut velocity, mut acceleration, mut expiration_time, mut size) =
        if protocol_version >= 42 {
            (
                read_tween_range_vec3(&mut reader)?,
                read_tween_range_vec3(&mut reader)?,
                read_tween_range_vec3(&mut reader)?,
                read_tween_range_f32(&mut reader)?,
                read_tween_range_f32(&mut reader)?,
            )
        } else {
            let position = read_range_vec3(&mut reader, false)?;
            let velocity = read_range_vec3(&mut reader, false)?;
            let acceleration = read_range_vec3(&mut reader, false)?;
            let expiration_time = read_range_f32(&mut reader, false)?;
            let size = read_range_f32(&mut reader, false)?;

            (
                ParticleTween {
                    style: 0,
                    repetitions: 1,
                    beginning: 0.0,
                    start: position,
                    end: position,
                },
                ParticleTween {
                    style: 0,
                    repetitions: 1,
                    beginning: 0.0,
                    start: velocity,
                    end: velocity,
                },
                ParticleTween {
                    style: 0,
                    repetitions: 1,
                    beginning: 0.0,
                    start: acceleration,
                    end: acceleration,
                },
                ParticleTween {
                    style: 0,
                    repetitions: 1,
                    beginning: 0.0,
                    start: expiration_time,
                    end: expiration_time,
                },
                ParticleTween {
                    style: 0,
                    repetitions: 1,
                    beginning: 0.0,
                    start: size,
                    end: size,
                },
            )
        };

    let collision_detection = read_particle_bool(&mut reader)?;
    let texture_name = read_particle_string(&mut reader)?;
    let id = reader.read_u32()?;
    let vertical = read_particle_bool(&mut reader)?;
    let collision_removal = read_particle_bool(&mut reader)?;
    let attached_id = reader.read_u16()?;
    let animation = read_animation(&mut reader)?;
    let glow = reader.read_u8()?;
    let object_collision = read_particle_bool(&mut reader)?;

    let mut node_param0 = 0;
    let mut node_param2 = 0;
    let mut node_tile = 0;
    let mut texture = ParticleTexture {
        name: texture_name,
        animated: false,
        blend_mode: ParticleBlendMode::Alpha,
        alpha: ParticleTween {
            style: 0,
            repetitions: 1,
            beginning: 0.0,
            start: 1.0,
            end: 1.0,
        },
        scale: ParticleTween {
            style: 0,
            repetitions: 1,
            beginning: 0.0,
            start: Vec2::ONE,
            end: Vec2::ONE,
        },
        animation: None,
    };
    let mut drag = ParticleTween {
        style: 0,
        repetitions: 1,
        beginning: 0.0,
        start: ParticleRangeVec3 {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
            bias: 0.0,
        },
        end: ParticleRangeVec3 {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
            bias: 0.0,
        },
    };
    let mut jitter = drag;
    let mut bounce = ParticleTween {
        style: 0,
        repetitions: 1,
        beginning: 0.0,
        start: ParticleRangeF32 {
            min: 0.0,
            max: 0.0,
            bias: 0.0,
        },
        end: ParticleRangeF32 {
            min: 0.0,
            max: 0.0,
            bias: 0.0,
        },
    };
    let mut attractor_kind = 0;
    let mut attract = None;
    let mut attractor_origin = None;
    let mut attractor_attachment = 0;
    let mut attractor_kill = true;
    let mut attractor_direction = None;
    let mut attractor_direction_attachment = 0;
    let mut radius = None;
    let mut texture_pool = Vec::new();

    if !reader.is_empty() {
        node_param0 = reader.read_u16()?;
        node_param2 = reader.read_u8()?;
        node_tile = reader.read_u8()?;

        if protocol_version < 42 && !reader.is_empty() {
            position.start.bias = read_finite_f32(&mut reader)?;
            velocity.start.bias = read_finite_f32(&mut reader)?;
            acceleration.start.bias = read_finite_f32(&mut reader)?;
            expiration_time.start.bias = read_finite_f32(&mut reader)?;
            size.start.bias = read_finite_f32(&mut reader)?;
            position.end = read_range_vec3(&mut reader, true)?;
            velocity.end = read_range_vec3(&mut reader, true)?;
            acceleration.end = read_range_vec3(&mut reader, true)?;
            expiration_time.end = read_range_f32(&mut reader, true)?;
            size.end = read_range_f32(&mut reader, true)?;
        }

        if !reader.is_empty() {
            texture = read_texture_properties(&mut reader, texture.name, true)?;
            drag = read_tween_range_vec3(&mut reader)?;
            jitter = read_tween_range_vec3(&mut reader)?;
            bounce = read_tween_range_f32(&mut reader)?;
            attractor_kind = reader.read_u8()?;

            if attractor_kind > 3 {
                return Err(invalid_particle("unsupported particle attractor kind"));
            }

            if attractor_kind != 0 {
                attract = Some(read_tween_range_f32(&mut reader)?);
                attractor_origin = Some(read_tween_vec3(&mut reader)?);
                attractor_attachment = reader.read_u16()?;
                attractor_kill = reader.read_u8()? & 1 != 0;

                if attractor_kind != 1 {
                    attractor_direction = Some(read_tween_vec3(&mut reader)?);
                    attractor_direction_attachment = reader.read_u16()?;
                }
            }

            radius = Some(read_tween_range_vec3(&mut reader)?);
            let texture_count = usize::from(reader.read_u16()?);

            if texture_count > reader.remaining() / 43 {
                return Err(invalid_particle(
                    "particle texture pool exceeds packet size",
                ));
            }

            texture_pool.reserve(texture_count);

            for _ in 0..texture_count {
                texture_pool.push(read_texture_pool_entry(&mut reader)?);
            }
        }
    }

    if !reader.is_empty() {
        return Err(invalid_particle("unexpected bytes after particle spawner"));
    }

    Ok(ParticleSpawnerParameters {
        id,
        attached_id,
        amount,
        time,
        common: ParticleCommonParameters {
            collision_detection,
            collision_removal,
            object_collision,
            vertical,
            texture,
            animation,
            glow,
            node_param0,
            node_param2,
            node_tile,
        },
        position,
        velocity,
        acceleration,
        expiration_time,
        size,
        drag,
        jitter,
        bounce,
        attractor_kind,
        attract,
        attractor_origin,
        attractor_attachment,
        attractor_kill,
        attractor_direction,
        attractor_direction_attachment,
        radius,
        texture_pool,
    })
}

pub fn decode_spawn_particle_batch(data: &[u8]) -> Result<Vec<ParticleParameters>, ProtocolError> {
    let mut packet_reader = PacketReader::new(data);
    let compressed = packet_reader.read_string32()?;

    if !packet_reader.is_empty() {
        return Err(invalid_particle(
            "unexpected bytes after compressed particle batch",
        ));
    }

    let decompressed = decompress_zstd_limited(compressed, MAX_PARTICLE_BATCH_SIZE)
        .map_err(|_| invalid_particle("invalid compressed particle batch"))?;
    let mut batch_reader = PacketReader::new(&decompressed);
    let mut particles = Vec::new();

    while !batch_reader.is_empty() {
        let particle_data = batch_reader.read_string32()?;
        particles.push(decode_spawn_particle(particle_data)?);
    }

    Ok(particles)
}
