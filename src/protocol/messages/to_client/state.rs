use crate::protocol::commands::{
    TOCLIENT_ADD_PARTICLESPAWNER, TOCLIENT_BREATH, TOCLIENT_CAMERA, TOCLIENT_CLOUD_PARAMS,
    TOCLIENT_CSM_RESTRICTION_FLAGS, TOCLIENT_DEATHSCREEN_LEGACY, TOCLIENT_DELETE_PARTICLESPAWNER,
    TOCLIENT_EYE_OFFSET, TOCLIENT_FADE_SOUND, TOCLIENT_FOV, TOCLIENT_HP, TOCLIENT_HUD_SET_FLAGS,
    TOCLIENT_HUD_SET_PARAM, TOCLIENT_HUDADD, TOCLIENT_HUDCHANGE, TOCLIENT_HUDRM,
    TOCLIENT_LOCAL_PLAYER_ANIMATIONS, TOCLIENT_MINIMAP_MODES, TOCLIENT_MOVE_PLAYER,
    TOCLIENT_MOVE_PLAYER_REL, TOCLIENT_MOVEMENT, TOCLIENT_OVERRIDE_DAY_NIGHT_RATIO,
    TOCLIENT_PLAY_SOUND, TOCLIENT_PLAYER_SPEED, TOCLIENT_PRIVILEGES, TOCLIENT_SET_LIGHTING,
    TOCLIENT_SET_MOON, TOCLIENT_SET_SKY, TOCLIENT_SET_STARS, TOCLIENT_SET_SUN,
    TOCLIENT_SPAWN_PARTICLE, TOCLIENT_SPAWN_PARTICLE_BATCH, TOCLIENT_STOP_SOUND,
    TOCLIENT_TIME_OF_DAY, TOCLIENT_UPDATE_PLAYER_LIST,
};
use crate::protocol::hud::{
    HudAdd, HudChange, decode_hud_add, decode_hud_change, decode_hud_remove,
};
use crate::protocol::messages::{DecodeMessage, DecodeMessageWithProtocolVersion, Message};
use crate::protocol::minimap::{MinimapModesUpdate, decode_minimap_modes};
use crate::protocol::particle::{
    ParticleParameters, ParticleSpawnerParameters, decode_add_particle_spawner,
    decode_spawn_particle, decode_spawn_particle_batch,
};
use crate::protocol::player::{
    LegacyDeathscreen, ServerPlayerMovement, decode_deathscreen_legacy, decode_move_player,
};
use crate::protocol::server_state::{
    CameraModeUpdate, CloudParamsUpdate, CsmRestrictionFlagsUpdate, DayNightRatioUpdate,
    DeleteParticleSpawner, EyeOffsetUpdate, FadeSoundUpdate, FovUpdate, HudFlagsUpdate,
    HudParamUpdate, LightingUpdate, LocalPlayerAnimationsUpdate, MoonUpdate, PlaySoundUpdate,
    PlayerListUpdate, PrivilegesUpdate, RelativePlayerMove, SkyUpdate, StarsUpdate,
    StopSoundUpdate, SunUpdate, decode_camera_mode, decode_cloud_params,
    decode_csm_restriction_flags, decode_day_night_ratio, decode_delete_particle_spawner,
    decode_eye_offset, decode_fade_sound, decode_fov, decode_hud_flags, decode_hud_param,
    decode_lighting, decode_local_player_animations, decode_moon, decode_play_sound,
    decode_player_list, decode_privileges, decode_relative_player_move, decode_sky, decode_stars,
    decode_stop_sound, decode_sun,
};
use crate::protocol::status::{
    HpUpdate, MovementParameters, PlayerSpeedUpdate, TimeOfDayUpdate, decode_movement_parameters,
    decode_player_breath, decode_player_hp, decode_player_speed, decode_time_of_day,
};

impl Message for MovementParameters {
    const COMMAND_ID: u16 = TOCLIENT_MOVEMENT;
}

impl DecodeMessage for MovementParameters {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_movement_parameters(payload)
    }
}

impl Message for TimeOfDayUpdate {
    const COMMAND_ID: u16 = TOCLIENT_TIME_OF_DAY;
}

impl DecodeMessage for TimeOfDayUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_time_of_day(payload)
    }
}

impl Message for HpUpdate {
    const COMMAND_ID: u16 = TOCLIENT_HP;
}

impl DecodeMessage for HpUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_player_hp(payload)
    }
}

impl Message for PlayerSpeedUpdate {
    const COMMAND_ID: u16 = TOCLIENT_PLAYER_SPEED;
}

impl DecodeMessage for PlayerSpeedUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_player_speed(payload)
    }
}

impl Message for LightingUpdate {
    const COMMAND_ID: u16 = TOCLIENT_SET_LIGHTING;
}

impl DecodeMessage for LightingUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_lighting(payload)
    }
}

impl Message for DayNightRatioUpdate {
    const COMMAND_ID: u16 = TOCLIENT_OVERRIDE_DAY_NIGHT_RATIO;
}

impl DecodeMessage for DayNightRatioUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_day_night_ratio(payload)
    }
}

impl Message for EyeOffsetUpdate {
    const COMMAND_ID: u16 = TOCLIENT_EYE_OFFSET;
}

impl DecodeMessage for EyeOffsetUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_eye_offset(payload)
    }
}

impl Message for CloudParamsUpdate {
    const COMMAND_ID: u16 = TOCLIENT_CLOUD_PARAMS;
}

impl DecodeMessage for CloudParamsUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_cloud_params(payload)
    }
}

impl Message for SunUpdate {
    const COMMAND_ID: u16 = TOCLIENT_SET_SUN;
}

impl DecodeMessage for SunUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_sun(payload)
    }
}

impl Message for MoonUpdate {
    const COMMAND_ID: u16 = TOCLIENT_SET_MOON;
}

impl DecodeMessage for MoonUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_moon(payload)
    }
}

impl Message for StarsUpdate {
    const COMMAND_ID: u16 = TOCLIENT_SET_STARS;
}

impl DecodeMessage for StarsUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_stars(payload)
    }
}

impl Message for PlaySoundUpdate {
    const COMMAND_ID: u16 = TOCLIENT_PLAY_SOUND;
}

impl DecodeMessage for PlaySoundUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_play_sound(payload)
    }
}

impl Message for StopSoundUpdate {
    const COMMAND_ID: u16 = TOCLIENT_STOP_SOUND;
}

impl DecodeMessage for StopSoundUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_stop_sound(payload)
    }
}

impl Message for FadeSoundUpdate {
    const COMMAND_ID: u16 = TOCLIENT_FADE_SOUND;
}

impl DecodeMessage for FadeSoundUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_fade_sound(payload)
    }
}

impl Message for PrivilegesUpdate {
    const COMMAND_ID: u16 = TOCLIENT_PRIVILEGES;
}

impl DecodeMessage for PrivilegesUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_privileges(payload)
    }
}

impl Message for PlayerListUpdate {
    const COMMAND_ID: u16 = TOCLIENT_UPDATE_PLAYER_LIST;
}

impl DecodeMessage for PlayerListUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_player_list(payload)
    }
}

impl Message for RelativePlayerMove {
    const COMMAND_ID: u16 = TOCLIENT_MOVE_PLAYER_REL;
}

impl DecodeMessage for RelativePlayerMove {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_relative_player_move(payload)
    }
}

impl Message for FovUpdate {
    const COMMAND_ID: u16 = TOCLIENT_FOV;
}

impl DecodeMessage for FovUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_fov(payload)
    }
}

impl Message for CsmRestrictionFlagsUpdate {
    const COMMAND_ID: u16 = TOCLIENT_CSM_RESTRICTION_FLAGS;
}

impl DecodeMessage for CsmRestrictionFlagsUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_csm_restriction_flags(payload)
    }
}

impl Message for CameraModeUpdate {
    const COMMAND_ID: u16 = TOCLIENT_CAMERA;
}

impl DecodeMessage for CameraModeUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_camera_mode(payload)
    }
}

impl Message for HudFlagsUpdate {
    const COMMAND_ID: u16 = TOCLIENT_HUD_SET_FLAGS;
}

impl DecodeMessage for HudFlagsUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_hud_flags(payload)
    }
}

impl Message for HudParamUpdate {
    const COMMAND_ID: u16 = TOCLIENT_HUD_SET_PARAM;
}

impl DecodeMessage for HudParamUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_hud_param(payload)
    }
}

impl Message for DeleteParticleSpawner {
    const COMMAND_ID: u16 = TOCLIENT_DELETE_PARTICLESPAWNER;
}

impl DecodeMessage for DeleteParticleSpawner {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_delete_particle_spawner(payload)
    }
}

impl Message for MinimapModesUpdate {
    const COMMAND_ID: u16 = TOCLIENT_MINIMAP_MODES;
}

impl DecodeMessage for MinimapModesUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_minimap_modes(payload)
    }
}

impl Message for ParticleParameters {
    const COMMAND_ID: u16 = TOCLIENT_SPAWN_PARTICLE;
}

impl DecodeMessage for ParticleParameters {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_spawn_particle(payload)
    }
}

impl Message for LegacyDeathscreen {
    const COMMAND_ID: u16 = TOCLIENT_DEATHSCREEN_LEGACY;
}

impl DecodeMessage for LegacyDeathscreen {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_deathscreen_legacy(payload)
    }
}

impl Message for ServerPlayerMovement {
    const COMMAND_ID: u16 = TOCLIENT_MOVE_PLAYER;
}

impl DecodeMessage for ServerPlayerMovement {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_move_player(payload)
    }
}

impl Message for SkyUpdate {
    const COMMAND_ID: u16 = TOCLIENT_SET_SKY;
}

impl DecodeMessageWithProtocolVersion for SkyUpdate {
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_sky(payload, protocol_version)
    }
}

impl Message for LocalPlayerAnimationsUpdate {
    const COMMAND_ID: u16 = TOCLIENT_LOCAL_PLAYER_ANIMATIONS;
}

impl DecodeMessageWithProtocolVersion for LocalPlayerAnimationsUpdate {
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_local_player_animations(payload, protocol_version)
    }
}

impl Message for HudAdd {
    const COMMAND_ID: u16 = TOCLIENT_HUDADD;
}

impl DecodeMessageWithProtocolVersion for HudAdd {
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_hud_add(payload, protocol_version)
    }
}

impl Message for HudChange {
    const COMMAND_ID: u16 = TOCLIENT_HUDCHANGE;
}

impl DecodeMessageWithProtocolVersion for HudChange {
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_hud_change(payload, protocol_version)
    }
}

impl Message for ParticleSpawnerParameters {
    const COMMAND_ID: u16 = TOCLIENT_ADD_PARTICLESPAWNER;
}

impl DecodeMessageWithProtocolVersion for ParticleSpawnerParameters {
    fn decode(data: &[u8], protocol_version: u16) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        decode_add_particle_spawner(payload, protocol_version)
    }
}

/// Server message containing the ID of the HUD element to remove.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudRemove {
    /// HUD element identifier.
    pub id: u32,
}

impl Message for HudRemove {
    const COMMAND_ID: u16 = TOCLIENT_HUDRM;
}

impl DecodeMessage for HudRemove {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let id = decode_hud_remove(payload)?;
        Ok(Self { id })
    }
}

/// Server message containing the player's breath value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerBreathUpdate {
    /// Current breath value.
    pub breath: u16,
}

impl Message for PlayerBreathUpdate {
    const COMMAND_ID: u16 = TOCLIENT_BREATH;
}

impl DecodeMessage for PlayerBreathUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let breath = decode_player_breath(payload)?;
        Ok(Self { breath })
    }
}

/// Server message containing a batch of particle parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleBatchUpdate {
    /// Particles included in the batch.
    pub particles: Vec<ParticleParameters>,
}

impl Message for ParticleBatchUpdate {
    const COMMAND_ID: u16 = TOCLIENT_SPAWN_PARTICLE_BATCH;
}

impl DecodeMessage for ParticleBatchUpdate {
    fn decode(data: &[u8]) -> Result<Self, crate::errors::ProtocolError> {
        let payload = <Self as Message>::decode_payload(data)?;
        let particles = decode_spawn_particle_batch(payload)?;
        Ok(Self { particles })
    }
}
