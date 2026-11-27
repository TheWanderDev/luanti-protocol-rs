use bytes::BufMut;
use std::hint::black_box;

use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
};
use luanti_protocol_rs::protocol::packet::{decode_datagram, encode_header};

fn packet_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("packet_decode");

    for size in [64_usize, 256, 512] {
        let mut datagram = encode_header(7, 0);
        datagram.put_u8(1);
        datagram.resize(size, 0x5A);
        let datagram = datagram.freeze();

        group.throughput(Throughput::Bytes(datagram.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("original", size),
            &datagram,
            |b, input| {
                b.iter_batched(
                    || input.clone(),
                    |packet| black_box(decode_datagram(black_box(packet))),
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, packet_decode);
criterion_main!(benches);
