#![no_main]

use libfuzzer_sys::fuzz_target;
use luanti_protocol_rs::fuzz::definitions;

fuzz_target!(|data: &[u8]| {
    definitions(data);
});
