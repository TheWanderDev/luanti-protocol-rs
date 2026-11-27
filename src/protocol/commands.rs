use num_enum::IntoPrimitive;
use strum::{Display, EnumIter, FromRepr};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter, FromRepr, IntoPrimitive)]
#[repr(u16)]
pub enum ToClientCommand {
    Hello = 0x02,
    AuthAccept = 0x03,
    AcceptSudoMode = 0x04,
    DenySudoMode = 0x05,
    AccessDenied = 0x0A,
    BlockData = 0x20,
    AddNode = 0x21,
    RemoveNode = 0x22,
    Inventory = 0x27,
    TimeOfDay = 0x29,
    CsmRestrictionFlags = 0x2A,
    PlayerSpeed = 0x2B,
    MediaPush = 0x2C,
    ChatMessage = 0x2F,
    ActiveObjectRemoveAdd = 0x31,
    ActiveObjectMessages = 0x32,
    Hp = 0x33,
    MovePlayer = 0x34,
    AccessDeniedLegacy = 0x35,
    Fov = 0x36,
    DeathscreenLegacy = 0x37,
    Media = 0x38,
    NodeDef = 0x3A,
    AnnounceMedia = 0x3C,
    ItemDef = 0x3D,
    PlaySound = 0x3F,
    StopSound = 0x40,
    Privileges = 0x41,
    InventoryFormspec = 0x42,
    DetachedInventory = 0x43,
    ShowFormspec = 0x44,
    Movement = 0x45,
    SpawnParticle = 0x46,
    AddParticleSpawner = 0x47,
    Camera = 0x48,
    HudAdd = 0x49,
    HudRemove = 0x4A,
    HudChange = 0x4B,
    HudSetFlags = 0x4C,
    HudSetParam = 0x4D,
    Breath = 0x4E,
    SetSky = 0x4F,
    OverrideDayNightRatio = 0x50,
    LocalPlayerAnimations = 0x51,
    EyeOffset = 0x52,
    DeleteParticleSpawner = 0x53,
    CloudParams = 0x54,
    FadeSound = 0x55,
    UpdatePlayerList = 0x56,
    ModChannelMessage = 0x57,
    ModChannelSignal = 0x58,
    NodeMetaChanged = 0x59,
    SetSun = 0x5A,
    SetMoon = 0x5B,
    SetStars = 0x5C,
    MovePlayerRelative = 0x5D,
    SrpBytesSB = 0x60,
    FormspecPrepend = 0x61,
    MinimapModes = 0x62,
    SetLighting = 0x63,
    SpawnParticleBatch = 0x64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter, FromRepr, IntoPrimitive)]
#[repr(u16)]
pub enum ToServerCommand {
    Init = 0x02,
    Init2 = 0x11,
    ModChannelJoin = 0x17,
    ModChannelLeave = 0x18,
    ModChannelMessage = 0x19,
    PlayerPosition = 0x23,
    GotBlocks = 0x24,
    DeletedBlocks = 0x25,
    InventoryAction = 0x31,
    ChatMessage = 0x32,
    Damage = 0x35,
    PlayerItem = 0x37,
    RespawnLegacy = 0x38,
    Interact = 0x39,
    RemovedSounds = 0x3A,
    NodeMetaFields = 0x3B,
    InventoryFields = 0x3C,
    RequestMedia = 0x40,
    HaveMedia = 0x41,
    ClientReady = 0x43,
    FirstSrp = 0x50,
    SrpBytesA = 0x51,
    SrpBytesM = 0x52,
    UpdateClientInfo = 0x53,
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

pub const TOCLIENT_NUM_MSG_TYPES: u16 = 0x65;
pub const TOSERVER_NUM_MSG_TYPES: u16 = 0x54;
