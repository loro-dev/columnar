#![no_main]
use columnar_fuzz::VecStore;
use libfuzzer_sys::fuzz_target;
use serde_columnar::{from_bytes, to_vec};

fuzz_target!(|store: VecStore| {
    // fuzzed code goes here
    if let Ok(buf) = to_vec(&store) {
        let store2 = from_bytes(&buf).unwrap();
        assert_eq!(store, store2);
    }
});