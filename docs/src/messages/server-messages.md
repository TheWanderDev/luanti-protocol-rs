# Server Messages

Server-to-client application message types are exported from `protocol::messages::to_client`.
Read the command ID with `protocol::messages::command_id` for dispatch, then pass the complete
application message bytes to the selected type's decode trait.

| Message family | Representative public types |
|---|---|
| Handshake and authentication | `Hello`, `AuthAccept`, `AccessDenied`, `SrpChallenge` |
| Chat | `ChatMessage` |
| World and definitions | `BlockDataUpdate`, `AddNodeUpdate`, `RemoveNodeUpdate`, `NodeDefinitionsUpdate`, `ItemDefinitionsUpdate` |
| Inventory | `InventoryUpdate`, `DetachedInventoryUpdate` |
| Media | `AnnounceMedia`, `MediaBunch`, `MediaPushUpdate` |
| Player and status | `ServerPlayerMovement`, `LegacyDeathscreen`, `MovementParameters`, `TimeOfDayUpdate`, `HpUpdate`, `PlayerSpeedUpdate`, `PlayerBreathUpdate` |
| HUD and environment | `HudAdd`, `HudRemove`, `HudChange`, `LightingUpdate`, `SkyUpdate`, `CameraModeUpdate`, `FovUpdate` |
| Entities and particles | `ActiveObjectRemoveAdd`, `ActiveObjectMessagesUpdate`, `ParticleParameters`, `ParticleSpawnerParameters`, `ParticleBatchUpdate` |
| Other state | `PrivilegesUpdate`, `PlayerListUpdate`, `MinimapModesUpdate`, `ModChannelSignal`, `ShowFormspec`, `FormspecPrepend` |

Use `DecodeMessage::decode` for ordinary messages. Use
`DecodeMessageWithProtocolVersion::decode(data, protocol_version)` or
`DecodeMessageWithSerializationVersion::decode(data, serialization_version)` when implemented by
the selected message type. The negotiated session version must be passed unchanged.
