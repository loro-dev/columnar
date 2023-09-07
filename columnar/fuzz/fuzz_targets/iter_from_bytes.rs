#![no_main]
use columnar_fuzz::VecStore;
use libfuzzer_sys::fuzz_target;
use serde_columnar::{iter_from_bytes, to_vec};

fuzz_target!(|store: VecStore| {
    // fuzzed code goes here
    let buf = to_vec(&store).unwrap();
    let _store2 = iter_from_bytes::<VecStore>(&buf).unwrap();
});
