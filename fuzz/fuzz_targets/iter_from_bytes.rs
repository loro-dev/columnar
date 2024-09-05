#![no_main]
use columnar_fuzz::Data;
use libfuzzer_sys::fuzz_target;
use serde_columnar::{iter_from_bytes, to_vec};

fuzz_target!(|store: Data| {
    // fuzzed code goes here
    if let Ok(buf) = to_vec(&store) {
        let _store2 = iter_from_bytes::<Data>(&buf).unwrap();
    }
});
