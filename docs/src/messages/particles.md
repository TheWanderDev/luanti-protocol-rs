# Particles

Particle payload models and decoders are in [`crate::protocol::particle`]. The module supports individual particle parameters, particle spawners, and batches.

[`crate::protocol::particle::ParticleParameters`] groups common rendering/collision settings with position, velocity, acceleration, drag, expiration time, size, jitter, and bounce ranges. [`crate::protocol::particle::ParticleSpawnerParameters`] adds an ID, attachment, emission amount and duration, tweened ranges, attractor configuration, and a texture pool.

`ParticleTween<T>`, `ParticleRangeVec3`, and `ParticleRangeF32` represent interpolation and numeric ranges. `ParticleTexture`, `ParticleAnimation`, and `ParticleBlendMode` describe texture presentation.

The public decoders are `decode_spawn_particle`, `decode_add_particle_spawner`, and `decode_spawn_particle_batch`. They validate the payload structure and return typed values; particle simulation and rendering are outside this crate.
