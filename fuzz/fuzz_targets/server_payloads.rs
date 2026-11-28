#![no_main]

use libfuzzer_sys::fuzz_target;
use luanti_protocol_rs::fuzz::server_payloads;

fuzz_target!(|data: &[u8]| {
    server_payloads(data);
});
