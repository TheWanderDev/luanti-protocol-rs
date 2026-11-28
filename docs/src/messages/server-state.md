# Server State Updates

Server state models are exported through `protocol::messages::to_client`. They represent decoded
updates; applying them to a world, player, audio system, or renderer is the caller's responsibility.

## Environment and audiovisual state

Exports include `LightingUpdate`, `SkyUpdate`, `CloudParamsUpdate`, `SunUpdate`, `MoonUpdate`,
`StarsUpdate`, `DayNightRatioUpdate`, `EyeOffsetUpdate`, `PlaySoundUpdate`, `StopSoundUpdate`, and
`FadeSoundUpdate`.

## Player and client state

Exports include `PrivilegesUpdate`, `PlayerListUpdate`, `RelativePlayerMove`,
`CsmRestrictionFlagsUpdate`, `CameraModeUpdate`, `FovUpdate`, `HudFlagsUpdate`, `HudParamUpdate`,
`LocalPlayerAnimationsUpdate`, and `DeleteParticleSpawner`.

## Version handling

Use the decode trait implemented by each message. Types with protocol-dependent wire layouts use
`DecodeMessageWithProtocolVersion`; pass the negotiated protocol version from the active session.
Do not substitute the crate's maximum supported version for the version negotiated with the server.
