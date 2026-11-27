use std::hint::black_box;
use std::io::Write;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use flate2::{Compression, write::ZlibEncoder};

fn compression(c: &mut Criterion) {
    let mut state = 0xA341_316C_u32;
    let mut input = Vec::with_capacity(256 * 1024);
    for _ in 0..256 * 1024 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        input.push(state.to_le_bytes()[0]);
    }

    let Ok(zstd) = zstd::encode_all(input.as_slice(), 3) else {
        return;
    };

    let mut zlib_encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    if zlib_encoder.write_all(&input).is_err() {
        return;
    }
    let Ok(zlib) = zlib_encoder.finish() else {
        return;
    };

    let mut group = c.benchmark_group("decompression");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input(BenchmarkId::new("zstd", input.len()), &zstd, |b, data| {
        b.iter(|| {
            black_box(
                luanti_protocol_rs::protocol::compression::decompress_zstd_limited(
                    black_box(data.as_slice()),
                    input.len(),
                ),
            )
        });
    });
    group.bench_with_input(BenchmarkId::new("zlib", input.len()), &zlib, |b, data| {
        b.iter(|| {
            black_box(
                luanti_protocol_rs::protocol::compression::decompress_zlib_frame_limited(
                    black_box(data.as_slice()),
                    input.len(),
                ),
            )
        });
    });
    group.finish();
}

criterion_group!(benches, compression);
criterion_main!(benches);
