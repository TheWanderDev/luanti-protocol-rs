# Server Messages

Server-to-client payload decoders are grouped by the data they produce. The command ID is normally dispatched by the caller before passing the command payload to the relevant decoder.

| Message family | Module | Representative decoder |
|---|---|---|
| Authentication | `messages` | `decode_auth_server_message` |
| Chat | `chat` | `decode_chat_message` |
| Player movement and death screen | `player` | `decode_move_player`, `decode_deathscreen_legacy` |
| Player status | `status` | `decode_movement_parameters`, `decode_time_of_day`, `decode_player_hp` |
| Environment and audio | `server_state` | `decode_lighting`, `decode_sky`, `decode_play_sound` |
| HUD | `hud` | `decode_hud_add`, `decode_hud_remove`, `decode_hud_change` |
| Particles | `particle` | `decode_spawn_particle`, `decode_add_particle_spawner` |
| Forms | `formspec` | `decode_show_formspec`, `decode_inventory_formspec` |
| Media | `media` | `decode_announce_media`, `decode_media_bunch`, `decode_media_push` |
| World and definitions | `mapblock`, `nodedef`, `itemdef`, `tiledef` | Domain-specific decoders |
| Entities | `active_object` | `decode_active_object_remove_add`, `decode_active_object_messages` |

Version-sensitive decoders accept the negotiated protocol version explicitly. Check each function signature in the [API reference](../api/luanti_protocol_rs/index.html) before calling it.
