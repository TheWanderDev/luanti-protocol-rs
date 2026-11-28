#![no_main]

use libfuzzer_sys::fuzz_target;
use luanti_protocol_rs::fuzz::active_objects;

fuzz_target!(|data: &[u8]| {
    active_objects(data);
});
