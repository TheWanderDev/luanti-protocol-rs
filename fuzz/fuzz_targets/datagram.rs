#![no_main]

use bytes::Bytes;
use libfuzzer_sys::fuzz_target;
use luanti_protocol_rs::protocol::packet::decode_datagram;

fuzz_target!(|data: &[u8]| {
    if data.len() <= 65_507 {
        let _ = decode_datagram(Bytes::copy_from_slice(data));
    }
});
