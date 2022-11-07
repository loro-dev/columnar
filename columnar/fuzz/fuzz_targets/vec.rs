#![no_main]
use columnar::{from_bytes, to_vec};
use columnar_fuzz::VecStore;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|store: VecStore| {
    // fuzzed code goes here
    let buf = to_vec(&store).unwrap();
    let store2 = from_bytes(&buf).unwrap();
    assert_eq!(store, store2);
});
