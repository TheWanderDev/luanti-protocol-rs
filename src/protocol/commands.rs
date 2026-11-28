use num_enum::IntoPrimitive;
use strum::{Display, EnumIter, FromRepr};

pub const TOCLIENT_HELLO: u16 = 0x02;
pub const TOCLIENT_AUTH_ACCEPT: u16 = 0x03;
pub const TOCLIENT_ACCEPT_SUDO_MODE: u16 = 0x04;
pub const TOCLIENT_DENY_SUDO_MODE: u16 = 0x05;
pub const TOCLIENT_ACCESS_DENIED: u16 = 0x0A;
pub const TOCLIENT_BLOCKDATA: u16 = 0x20;
pub const TOCLIENT_ADDNODE: u16 = 0x21;
pub const TOCLIENT_REMOVENODE: u16 = 0x22;
pub const TOCLIENT_INVENTORY: u16 = 0x27;
pub const TOCLIENT_TIME_OF_DAY: u16 = 0x29;
pub const TOCLIENT_CSM_RESTRICTION_FLAGS: u16 = 0x2A;
pub const TOCLIENT_PLAYER_SPEED: u16 = 0x2B;
pub const TOCLIENT_MEDIA_PUSH: u16 = 0x2C;
pub const TOCLIENT_CHAT_MESSAGE: u16 = 0x2F;
pub const TOCLIENT_ACTIVE_OBJECT_REMOVE_ADD: u16 = 0x31;
pub const TOCLIENT_ACTIVE_OBJECT_MESSAGES: u16 = 0x32;
pub const TOCLIENT_HP: u16 = 0x33;
pub const TOCLIENT_MOVE_PLAYER: u16 = 0x34;
pub const TOCLIENT_ACCESS_DENIED_LEGACY: u16 = 0x35;
pub const TOCLIENT_FOV: u16 = 0x36;
pub const TOCLIENT_DEATHSCREEN_LEGACY: u16 = 0x37;
pub const TOCLIENT_MEDIA: u16 = 0x38;
pub const TOCLIENT_NODEDEF: u16 = 0x3A;
pub const TOCLIENT_ANNOUNCE_MEDIA: u16 = 0x3C;
pub const TOCLIENT_ITEMDEF: u16 = 0x3D;
pub const TOCLIENT_PLAY_SOUND: u16 = 0x3F;
pub const TOCLIENT_STOP_SOUND: u16 = 0x40;
pub const TOCLIENT_PRIVILEGES: u16 = 0x41;
pub const TOCLIENT_INVENTORY_FORMSPEC: u16 = 0x42;
pub const TOCLIENT_DETACHED_INVENTORY: u16 = 0x43;
pub const TOCLIENT_SHOW_FORMSPEC: u16 = 0x44;
pub const TOCLIENT_MOVEMENT: u16 = 0x45;
pub const TOCLIENT_SPAWN_PARTICLE: u16 = 0x46;
pub const TOCLIENT_ADD_PARTICLESPAWNER: u16 = 0x47;
pub const TOCLIENT_CAMERA: u16 = 0x48;
pub const TOCLIENT_HUDADD: u16 = 0x49;
pub const TOCLIENT_HUDRM: u16 = 0x4A;
pub const TOCLIENT_HUDCHANGE: u16 = 0x4B;
pub const TOCLIENT_HUD_SET_FLAGS: u16 = 0x4C;
pub const TOCLIENT_HUD_SET_PARAM: u16 = 0x4D;
pub const TOCLIENT_BREATH: u16 = 0x4E;
pub const TOCLIENT_SET_SKY: u16 = 0x4F;
pub const TOCLIENT_OVERRIDE_DAY_NIGHT_RATIO: u16 = 0x50;
pub const TOCLIENT_LOCAL_PLAYER_ANIMATIONS: u16 = 0x51;
pub const TOCLIENT_EYE_OFFSET: u16 = 0x52;
pub const TOCLIENT_DELETE_PARTICLESPAWNER: u16 = 0x53;
pub const TOCLIENT_CLOUD_PARAMS: u16 = 0x54;
pub const TOCLIENT_FADE_SOUND: u16 = 0x55;
pub const TOCLIENT_UPDATE_PLAYER_LIST: u16 = 0x56;
pub const TOCLIENT_MODCHANNEL_MSG: u16 = 0x57;
pub const TOCLIENT_MODCHANNEL_SIGNAL: u16 = 0x58;
pub const TOCLIENT_NODEMETA_CHANGED: u16 = 0x59;
pub const TOCLIENT_SET_SUN: u16 = 0x5A;
pub const TOCLIENT_SET_MOON: u16 = 0x5B;
pub const TOCLIENT_SET_STARS: u16 = 0x5C;
pub const TOCLIENT_MOVE_PLAYER_REL: u16 = 0x5D;
pub const TOCLIENT_SRP_BYTES_SB: u16 = 0x60;
pub const TOCLIENT_FORMSPEC_PREPEND: u16 = 0x61;
pub const TOCLIENT_MINIMAP_MODES: u16 = 0x62;
pub const TOCLIENT_SET_LIGHTING: u16 = 0x63;
pub const TOCLIENT_SPAWN_PARTICLE_BATCH: u16 = 0x64;
pub const TOCLIENT_NUM_MSG_TYPES: u16 = 0x65;

pub const TOSERVER_INIT: u16 = 0x02;
pub const TOSERVER_INIT2: u16 = 0x11;
pub const TOSERVER_MODCHANNEL_JOIN: u16 = 0x17;
pub const TOSERVER_MODCHANNEL_LEAVE: u16 = 0x18;
pub const TOSERVER_MODCHANNEL_MSG: u16 = 0x19;
pub const TOSERVER_PLAYERPOS: u16 = 0x23;
pub const TOSERVER_GOTBLOCKS: u16 = 0x24;
pub const TOSERVER_DELETEDBLOCKS: u16 = 0x25;
pub const TOSERVER_INVENTORY_ACTION: u16 = 0x31;
pub const TOSERVER_CHAT_MESSAGE: u16 = 0x32;
pub const TOSERVER_DAMAGE: u16 = 0x35;
pub const TOSERVER_PLAYERITEM: u16 = 0x37;
pub const TOSERVER_RESPAWN_LEGACY: u16 = 0x38;
pub const TOSERVER_INTERACT: u16 = 0x39;
pub const TOSERVER_REMOVED_SOUNDS: u16 = 0x3A;
pub const TOSERVER_NODEMETA_FIELDS: u16 = 0x3B;
pub const TOSERVER_INVENTORY_FIELDS: u16 = 0x3C;
pub const TOSERVER_REQUEST_MEDIA: u16 = 0x40;
pub const TOSERVER_HAVE_MEDIA: u16 = 0x41;
pub const TOSERVER_CLIENT_READY: u16 = 0x43;
pub const TOSERVER_FIRST_SRP: u16 = 0x50;
pub const TOSERVER_SRP_BYTES_A: u16 = 0x51;
pub const TOSERVER_SRP_BYTES_M: u16 = 0x52;
pub const TOSERVER_UPDATE_CLIENT_INFO: u16 = 0x53;
pub const TOSERVER_NUM_MSG_TYPES: u16 = 0x54;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter, FromRepr, IntoPrimitive)]
#[repr(u16)]
pub enum ToClientCommand {
    Hello = TOCLIENT_HELLO,
    AuthAccept = TOCLIENT_AUTH_ACCEPT,
    AcceptSudoMode = TOCLIENT_ACCEPT_SUDO_MODE,
    DenySudoMode = TOCLIENT_DENY_SUDO_MODE,
    AccessDenied = TOCLIENT_ACCESS_DENIED,
    BlockData = TOCLIENT_BLOCKDATA,
    AddNode = TOCLIENT_ADDNODE,
    RemoveNode = TOCLIENT_REMOVENODE,
    Inventory = TOCLIENT_INVENTORY,
    TimeOfDay = TOCLIENT_TIME_OF_DAY,
    CsmRestrictionFlags = TOCLIENT_CSM_RESTRICTION_FLAGS,
    PlayerSpeed = TOCLIENT_PLAYER_SPEED,
    MediaPush = TOCLIENT_MEDIA_PUSH,
    ChatMessage = TOCLIENT_CHAT_MESSAGE,
    ActiveObjectRemoveAdd = TOCLIENT_ACTIVE_OBJECT_REMOVE_ADD,
    ActiveObjectMessages = TOCLIENT_ACTIVE_OBJECT_MESSAGES,
    Hp = TOCLIENT_HP,
    MovePlayer = TOCLIENT_MOVE_PLAYER,
    AccessDeniedLegacy = TOCLIENT_ACCESS_DENIED_LEGACY,
    Fov = TOCLIENT_FOV,
    DeathscreenLegacy = TOCLIENT_DEATHSCREEN_LEGACY,
    Media = TOCLIENT_MEDIA,
    NodeDef = TOCLIENT_NODEDEF,
    AnnounceMedia = TOCLIENT_ANNOUNCE_MEDIA,
    ItemDef = TOCLIENT_ITEMDEF,
    PlaySound = TOCLIENT_PLAY_SOUND,
    StopSound = TOCLIENT_STOP_SOUND,
    Privileges = TOCLIENT_PRIVILEGES,
    InventoryFormspec = TOCLIENT_INVENTORY_FORMSPEC,
    DetachedInventory = TOCLIENT_DETACHED_INVENTORY,
    ShowFormspec = TOCLIENT_SHOW_FORMSPEC,
    Movement = TOCLIENT_MOVEMENT,
    SpawnParticle = TOCLIENT_SPAWN_PARTICLE,
    AddParticleSpawner = TOCLIENT_ADD_PARTICLESPAWNER,
    Camera = TOCLIENT_CAMERA,
    HudAdd = TOCLIENT_HUDADD,
    HudRemove = TOCLIENT_HUDRM,
    HudChange = TOCLIENT_HUDCHANGE,
    HudSetFlags = TOCLIENT_HUD_SET_FLAGS,
    HudSetParam = TOCLIENT_HUD_SET_PARAM,
    Breath = TOCLIENT_BREATH,
    SetSky = TOCLIENT_SET_SKY,
    OverrideDayNightRatio = TOCLIENT_OVERRIDE_DAY_NIGHT_RATIO,
    LocalPlayerAnimations = TOCLIENT_LOCAL_PLAYER_ANIMATIONS,
    EyeOffset = TOCLIENT_EYE_OFFSET,
    DeleteParticleSpawner = TOCLIENT_DELETE_PARTICLESPAWNER,
    CloudParams = TOCLIENT_CLOUD_PARAMS,
    FadeSound = TOCLIENT_FADE_SOUND,
    UpdatePlayerList = TOCLIENT_UPDATE_PLAYER_LIST,
    ModChannelMessage = TOCLIENT_MODCHANNEL_MSG,
    ModChannelSignal = TOCLIENT_MODCHANNEL_SIGNAL,
    NodeMetaChanged = TOCLIENT_NODEMETA_CHANGED,
    SetSun = TOCLIENT_SET_SUN,
    SetMoon = TOCLIENT_SET_MOON,
    SetStars = TOCLIENT_SET_STARS,
    MovePlayerRelative = TOCLIENT_MOVE_PLAYER_REL,
    SrpBytesSB = TOCLIENT_SRP_BYTES_SB,
    FormspecPrepend = TOCLIENT_FORMSPEC_PREPEND,
    MinimapModes = TOCLIENT_MINIMAP_MODES,
    SetLighting = TOCLIENT_SET_LIGHTING,
    SpawnParticleBatch = TOCLIENT_SPAWN_PARTICLE_BATCH,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter, FromRepr, IntoPrimitive)]
#[repr(u16)]
pub enum ToServerCommand {
    Init = TOSERVER_INIT,
    Init2 = TOSERVER_INIT2,
    ModChannelJoin = TOSERVER_MODCHANNEL_JOIN,
    ModChannelLeave = TOSERVER_MODCHANNEL_LEAVE,
    ModChannelMessage = TOSERVER_MODCHANNEL_MSG,
    PlayerPosition = TOSERVER_PLAYERPOS,
    GotBlocks = TOSERVER_GOTBLOCKS,
    DeletedBlocks = TOSERVER_DELETEDBLOCKS,
    InventoryAction = TOSERVER_INVENTORY_ACTION,
    ChatMessage = TOSERVER_CHAT_MESSAGE,
    Damage = TOSERVER_DAMAGE,
    PlayerItem = TOSERVER_PLAYERITEM,
    RespawnLegacy = TOSERVER_RESPAWN_LEGACY,
    Interact = TOSERVER_INTERACT,
    RemovedSounds = TOSERVER_REMOVED_SOUNDS,
    NodeMetaFields = TOSERVER_NODEMETA_FIELDS,
    InventoryFields = TOSERVER_INVENTORY_FIELDS,
    RequestMedia = TOSERVER_REQUEST_MEDIA,
    HaveMedia = TOSERVER_HAVE_MEDIA,
    ClientReady = TOSERVER_CLIENT_READY,
    FirstSrp = TOSERVER_FIRST_SRP,
    SrpBytesA = TOSERVER_SRP_BYTES_A,
    SrpBytesM = TOSERVER_SRP_BYTES_M,
    UpdateClientInfo = TOSERVER_UPDATE_CLIENT_INFO,
}

impl ToClientCommand {
    /// Returns the MTP channel used by the upstream server command factory.
    #[must_use]
    pub const fn channel(self) -> u8 {
        match self {
            Self::BlockData | Self::Media => 2,
            Self::HudAdd
            | Self::HudRemove
            | Self::HudChange
            | Self::HudSetFlags
            | Self::HudSetParam => 1,
            _ => 0,
        }
    }
}

impl ToServerCommand {
    #[must_use]
    pub const fn channel(self) -> u8 {
        match self {
            Self::Init
            | Self::Init2
            | Self::RequestMedia
            | Self::ClientReady
            | Self::FirstSrp
            | Self::SrpBytesA
            | Self::SrpBytesM => 1,
            Self::GotBlocks
            | Self::DeletedBlocks
            | Self::RemovedSounds
            | Self::HaveMedia
            | Self::UpdateClientInfo => 2,
            Self::ModChannelJoin
            | Self::ModChannelLeave
            | Self::ModChannelMessage
            | Self::PlayerPosition
            | Self::InventoryAction
            | Self::ChatMessage
            | Self::Damage
            | Self::PlayerItem
            | Self::RespawnLegacy
            | Self::Interact
            | Self::NodeMetaFields
            | Self::InventoryFields => 0,
        }
    }

    #[must_use]
    pub const fn is_reliable(self) -> bool {
        !matches!(self, Self::Init | Self::PlayerPosition)
    }
}
