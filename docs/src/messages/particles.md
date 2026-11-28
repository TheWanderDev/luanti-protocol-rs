# Particles

Particle server updates are exported from `protocol::messages::to_client`.

Public message types include `ParticleParameters`, `ParticleSpawnerParameters`,
`ParticleBatchUpdate`, and `DeleteParticleSpawner`. Use the decode trait implemented by the
selected type to parse a complete application message, including its command ID.

Particle models and supporting types are defined in `protocol::particle` and
`protocol::server_state`. The crate parses particle parameters and updates; simulation, resource
loading, and rendering belong to the client.
