use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};

fn mapblock_packet(populated: bool) -> Option<Vec<u8>> {
    let mut body = Vec::with_capacity(5 + 4096 * 4 + 3);
    body.extend_from_slice(&[0, 0, 0, 2, 2]);

    for index in 0_u16..4096 {
        let content_id = if populated { index % 255 + 1 } else { 0 };
        body.extend_from_slice(&content_id.to_be_bytes());
    }

    body.resize(5 + 4096 * 4, u8::from(populated));
    body.extend_from_slice(&[2, 0, 0]);

    let compressed = zstd::encode_all(body.as_slice(), 3).ok()?;
    let mut packet = Vec::with_capacity(7 + compressed.len());
    packet.extend_from_slice(&[0; 6]);
    packet.extend_from_slice(&compressed);
    packet.push(0);

    Some(packet)
}

fn mapblock_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("mapblock_decode");

    for (name, populated) in [("zeroed_nodes", false), ("populated_nodes", true)] {
        let Some(packet) = mapblock_packet(populated) else {
            return;
        };

        group.throughput(Throughput::Bytes(packet.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(luanti_protocol_rs::protocol::mapblock::decode_map_block(
                    black_box(packet.as_slice()),
                    29,
                ))
            });
        });
    }

    group.finish();
}

criterion_group!(benches, mapblock_decode);
criterion_main!(benches);
